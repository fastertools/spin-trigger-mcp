use spin_mcp_sdk::{mcp_component, Request, Response, Tool, ToolResult, Error};
use serde_json::json;

#[mcp_component]
fn handle_request(request: Request) -> Response {
    match request {
        Request::ToolsList => {
            // TODO: Define your tools here
            Response::ToolsList(vec![
                Tool {
                    name: "example_tool".to_string(),
                    description: "An example tool that echoes input".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "Message to echo"
                            }
                        },
                        "required": ["message"]
                    }).to_string(),
                }
            ])
        }
        
        Request::ToolsCall(params) => {
            match params.name.as_str() {
                "example_tool" => {
                    // Parse arguments
                    let args: serde_json::Value = serde_json::from_str(&params.arguments)
                        .unwrap_or(json!({}));
                    
                    let message = args.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No message provided");
                    
                    Response::ToolsCall(ToolResult::Text(
                        format!("Echo: {}", message)
                    ))
                }
                _ => Response::ToolsCall(ToolResult::Error(Error {
                    code: -32602,
                    message: format!("Unknown tool: {}", params.name),
                    data: None,
                }))
            }
        }
        
        Request::ResourcesList => {
            // TODO: Add resources if needed
            Response::ResourcesList(vec![])
        }
        
        Request::PromptsList => {
            // TODO: Add prompts if needed
            Response::PromptsList(vec![])
        }
        
        Request::Ping => Response::Pong,
        
        _ => Response::Error(Error {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        })
    }
}