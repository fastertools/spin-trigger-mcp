use serde::{Deserialize, Serialize};
use serde_json::json;

wit_bindgen::generate!({
    world: "{{project-name | snake_case}}-world",
    path: "wit",
});

struct Component;

impl exports::spin::mcp_trigger::mcp_types::Guest for Component {
    fn handle_request(
        request: exports::spin::mcp_trigger::mcp_types::Request,
    ) -> exports::spin::mcp_trigger::mcp_types::Response {
        use exports::spin::mcp_trigger::mcp_types::*;
        
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
    
    fn initialize() -> Result<(), String> {
        // TODO: Add initialization logic if needed
        Ok(())
    }
}