use anyhow::{Context, Result};
use base64::Engine;
use clap::Args;
use http::{Request as HttpRequest, Response as HttpResponse, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use spin_app::App;
use spin_factors::RuntimeFactors;
use spin_trigger::{Trigger, TriggerApp};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task;
use tracing::{info, warn};

// Generate bindings from WIT file
wasmtime::component::bindgen!({
    path: ".",
    world: "spin-mcp",
    async: true,
});

use spin::mcp_trigger::mcp_types as mcp;

/// MCP trigger implementation
#[derive(Clone)]
pub struct McpTrigger {
    listen_addr: SocketAddr,
    component_routes: HashMap<String, String>,
}

impl<F: RuntimeFactors> Trigger<F> for McpTrigger {
    const TYPE: &'static str = "mcp";
    type InstanceState = ();
    type CliArgs = CliArgs;

    fn new(cli_args: Self::CliArgs, app: &App) -> Result<Self> {
        let trigger_type = <Self as Trigger<F>>::TYPE;
        
        // Get trigger metadata if any
        let metadata = app
            .get_trigger_metadata::<TriggerMetadata>(trigger_type)?
            .unwrap_or_default();

        // Collect all MCP trigger configs
        let mut component_routes = HashMap::new();
        let configs = app.trigger_configs::<ComponentConfig>(trigger_type)?;
        
        for (component_id, config) in configs {
            info!("Registering MCP route {} -> component {} (id: {})", config.route, config.component, component_id);
            component_routes.insert(config.route.clone(), config.component.clone());
        }

        if component_routes.is_empty() {
            anyhow::bail!("No MCP components found in application");
        }

        info!("Found {} MCP component(s)", component_routes.len());

        Ok(Self {
            listen_addr: cli_args.address.unwrap_or(metadata.address),
            component_routes,
        })
    }

    async fn run(self, trigger_app: TriggerApp<Self, F>) -> Result<()> {
        let server = Arc::new(McpServer::new(self, trigger_app)?);
        server.serve().await
    }
}

/// CLI arguments for the MCP trigger
#[derive(Args, Debug)]
pub struct CliArgs {
    /// IP address and port to listen on
    #[clap(long = "listen", env = "SPIN_MCP_LISTEN_ADDR")]
    pub address: Option<SocketAddr>,

    /// Run a test request against each MCP component
    #[clap(long)]
    pub test: bool,
}

/// Trigger-level metadata (optional)
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerMetadata {
    /// Default address to listen on
    #[serde(default = "default_address")]
    pub address: SocketAddr,
}

impl Default for TriggerMetadata {
    fn default() -> Self {
        Self {
            address: default_address(),
        }
    }
}

fn default_address() -> SocketAddr {
    "127.0.0.1:3000".parse().unwrap()
}

/// Per-component configuration
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentConfig {
    /// The component ID
    pub component: String,
    /// The route path (e.g., "/weather")
    pub route: String,
}

/// MCP server that handles HTTP transport
struct McpServer<F: RuntimeFactors> {
    trigger: McpTrigger,
    trigger_app: Arc<TriggerApp<McpTrigger, F>>,
}

impl<F: RuntimeFactors> McpServer<F> {
    fn new(trigger: McpTrigger, trigger_app: TriggerApp<McpTrigger, F>) -> Result<Self> {
        Ok(Self {
            trigger,
            trigger_app: Arc::new(trigger_app),
        })
    }

    async fn serve(self: Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(self.trigger.listen_addr).await?;
        let actual_addr = listener.local_addr()?;
        
        info!("MCP trigger listening on http://{}", actual_addr);

        loop {
            let (stream, client_addr) = listener.accept().await?;
            let server = self.clone();
            
            task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        TokioIo::new(stream),
                        service_fn(move |request| {
                            server.clone().handle_http_request(client_addr, request)
                        }),
                    )
                    .await
                {
                    warn!("Error serving MCP connection: {err:?}");
                }
            });
        }
    }

    async fn handle_http_request(
        self: Arc<Self>,
        _client_addr: SocketAddr,
        mut req: HttpRequest<Incoming>,
    ) -> Result<HttpResponse<Full<Bytes>>> {
        let path = req.uri().path();
        
        // Find component for this route
        let component_id = self
            .trigger
            .component_routes
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("No MCP component found for route: {}", path))?;

        // Only accept POST requests
        if req.method() != http::Method::POST {
            return Ok(HttpResponse::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::new(Bytes::new()))?);
        }

        // Read request body
        let body_bytes = req.body_mut().collect().await?.to_bytes();
        
        // Parse JSON-RPC request
        let json_rpc_request: JsonRpcRequest = serde_json::from_slice(&body_bytes)
            .context("Failed to parse JSON-RPC request")?;

        // Handle the request
        let response = self
            .handle_mcp_request(component_id, json_rpc_request)
            .await?;

        // For notifications (no id), return empty response
        if response.is_none() {
            return Ok(HttpResponse::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Full::new(Bytes::new()))?);
        }

        // Serialize response
        let response_bytes = serde_json::to_vec(&response.unwrap())?;
        
        Ok(HttpResponse::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Full::new(response_bytes.into()))?)
    }

    async fn handle_mcp_request(
        &self,
        component_id: &str,
        json_rpc_request: JsonRpcRequest,
    ) -> Result<Option<JsonRpcResponse>> {
        // Convert JSON-RPC method to MCP request type
        let mcp_request = match json_rpc_request.method.as_str() {
            "tools/list" => mcp::Request::ToolsList,
            "tools/call" => {
                let params: ToolsCallParams = serde_json::from_value(json_rpc_request.params.unwrap_or_default())?;
                mcp::Request::ToolsCall(mcp::ToolsCallParams {
                    name: params.name,
                    arguments: serde_json::to_string(&params.arguments)?,
                })
            }
            "resources/list" => mcp::Request::ResourcesList,
            "resources/read" => {
                let params: ResourceReadParams = serde_json::from_value(json_rpc_request.params.unwrap_or_default())?;
                mcp::Request::ResourcesRead(mcp::ResourceReadParams { uri: params.uri })
            }
            "prompts/list" => mcp::Request::PromptsList,
            "prompts/get" => {
                let params: PromptGetParams = serde_json::from_value(json_rpc_request.params.unwrap_or_default())?;
                mcp::Request::PromptsGet(mcp::PromptGetParams {
                    name: params.name,
                    arguments: serde_json::to_string(&params.arguments)?,
                })
            }
            "ping" => mcp::Request::Ping,
            "initialize" => {
                // Handle MCP initialization handshake
                info!("Handling initialize request");
                // Initialize requests must have an ID
                if let Some(id) = json_rpc_request.id {
                    return Ok(Some(JsonRpcResponse::success(
                        id,
                        serde_json::json!({
                            "protocolVersion": "2025-03-26",
                            "capabilities": {
                                "tools": {},
                                "resources": {},
                                "prompts": {}
                            },
                            "serverInfo": {
                                "name": "spin-mcp-server",
                                "version": "0.1.0"
                            }
                        })
                    )));
                } else {
                    return Ok(None);
                }
            }
            "notifications/initialized" => {
                // Client acknowledging initialization - this is a notification
                info!("Client initialized notification received");
                return Ok(None); // Notifications don't get responses
            }
            _ => {
                warn!("Unknown method requested: {}", json_rpc_request.method);
                if let Some(id) = json_rpc_request.id {
                    return Ok(Some(JsonRpcResponse::error(
                        id,
                        -32601,
                        &format!("Method not found: {}", json_rpc_request.method),
                        None,
                    )));
                } else {
                    // Unknown notification - just ignore
                    return Ok(None);
                }
            }
        };

        // Prepare and instantiate the component
        let instance_builder = self.trigger_app.prepare(component_id)?;
        let (instance, mut store) = instance_builder.instantiate(()).await?;
        let instance = SpinMcp::new(&mut store, &instance)?;

        // Call the component's handler
        let mcp_response = instance
            .call_handle_request(&mut store, &mcp_request)
            .await?;

        // All component responses need an ID
        if let Some(id) = json_rpc_request.id {
            let json_rpc_response = match mcp_response {
                mcp::Response::ToolsList(tools) => {
                    let tools_json: Vec<_> = tools.into_iter().map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": serde_json::from_str::<serde_json::Value>(&t.input_schema).unwrap_or(serde_json::json!({})),
                        })
                    }).collect();
                    JsonRpcResponse::success(id.clone(), serde_json::json!({ "tools": tools_json }))
                }
                mcp::Response::ToolsCall(result) => {
                    match result {
                        mcp::ToolResult::Text(text) => {
                            JsonRpcResponse::success(id.clone(), serde_json::json!({ 
                                "content": [{
                                    "type": "text",
                                    "text": text
                                }]
                            }))
                        }
                        mcp::ToolResult::Json(json_str) => {
                            let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
                            JsonRpcResponse::success(id.clone(), serde_json::json!({ 
                                "content": [{
                                    "type": "text",
                                    "text": json_str
                                }]
                            }))
                        }
                        mcp::ToolResult::Binary(bytes) => {
                            use base64::Engine;
                            let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            JsonRpcResponse::success(id.clone(), serde_json::json!({ 
                                "content": [{
                                    "type": "image",
                                    "data": base64,
                                    "mimeType": "application/octet-stream"
                                }]
                            }))
                        }
                        mcp::ToolResult::Error(err) => {
                            // For errors, we return isError: true with content array
                            JsonRpcResponse::success(id.clone(), serde_json::json!({ 
                                "content": [{
                                    "type": "text",
                                    "text": err.message
                                }],
                                "isError": true
                            }))
                        }
                    }
                }
                mcp::Response::ResourcesList(resources) => {
                    let resources_json: Vec<_> = resources.into_iter().map(|r| {
                        serde_json::json!({
                            "uri": r.uri,
                            "name": r.name,
                            "description": r.description,
                            "mimeType": r.mime_type,
                        })
                    }).collect();
                    JsonRpcResponse::success(id.clone(), serde_json::json!({ "resources": resources_json }))
                }
                mcp::Response::ResourcesRead(contents) => {
                    JsonRpcResponse::success(id.clone(), serde_json::json!({
                        "uri": contents.uri,
                        "mimeType": contents.mime_type,
                        "text": contents.text,
                        "blob": contents.blob.map(|b| base64::engine::general_purpose::STANDARD.encode(&b)),
                    }))
                }
                mcp::Response::PromptsList(prompts) => {
                    let prompts_json: Vec<_> = prompts.into_iter().map(|p| {
                        serde_json::json!({
                            "name": p.name,
                            "description": p.description,
                            "arguments": p.arguments.into_iter().map(|a| {
                                serde_json::json!({
                                    "name": a.name,
                                    "description": a.description,
                                    "required": a.required,
                                })
                            }).collect::<Vec<_>>(),
                        })
                    }).collect();
                    JsonRpcResponse::success(id.clone(), serde_json::json!({ "prompts": prompts_json }))
                }
                mcp::Response::PromptsGet(messages) => {
                    let messages_json: Vec<_> = messages.into_iter().map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content,
                        })
                    }).collect();
                    JsonRpcResponse::success(id.clone(), serde_json::json!({ "messages": messages_json }))
                }
                mcp::Response::Pong => {
                    JsonRpcResponse::success(id.clone(), serde_json::json!("pong"))
                }
                mcp::Response::Error(err) => {
                    JsonRpcResponse::error(id.clone(), err.code, &err.message, err.data)
                }
                _ => {
                    JsonRpcResponse::success(id.clone(), serde_json::json!({}))
                }
            };
            Ok(Some(json_rpc_response))
        } else {
            // Component methods should have an ID, this is an error
            warn!("Component method called without request ID");
            Ok(None)
        }
    }
}

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i32, message: &str, data: Option<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: data.map(serde_json::Value::String),
            }),
            id,
        }
    }
}

// Parameter structures for JSON-RPC methods
#[derive(Debug, Deserialize)]
struct ToolsCallParams {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ResourceReadParams {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct PromptGetParams {
    name: String,
    arguments: serde_json::Value,
}