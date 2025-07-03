use spin_mcp_sdk::{Tool, ToolResult, Error};
use serde_json::json;

pub fn get_tools_list() -> Vec<Tool> {
    vec![
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
    ]
}

pub fn handle_tool_call(name: &str, arguments: &str) -> ToolResult {
    match name {
        "example_tool" => handle_example_tool(arguments),
        _ => ToolResult::Error(Error {
            code: -32602,
            message: format!("Unknown tool: {}", name),
            data: None,
        })
    }
}

fn handle_example_tool(arguments: &str) -> ToolResult {
    let args: serde_json::Value = serde_json::from_str(arguments)
        .unwrap_or(json!({}));
    
    let message = args.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("No message provided");
    
    ToolResult::Text(format!("Echo: {}", message))
}