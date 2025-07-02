# Demo MCP Server

This is a simple example of an MCP (Model Context Protocol) server built with the Spin MCP trigger plugin.

## Overview

This demo implements a basic echo tool that demonstrates the core MCP functionality:
- Lists available tools via `tools/list`
- Executes tools via `tools/call`
- Responds to health checks via `ping`

## Running the Example

1. Build the component:
   ```bash
   spin build
   ```

2. Run the server:
   ```bash
   spin up
   ```

   The server will start on `http://localhost:3000/mcp`

## Testing the Server

### List Available Tools

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "description": "An example tool that echoes input",
        "inputSchema": {
          "properties": {
            "message": {
              "description": "Message to echo",
              "type": "string"
            }
          },
          "required": ["message"],
          "type": "object"
        },
        "name": "example_tool"
      }
    ]
  },
  "id": 1
}
```

### Call the Echo Tool

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "example_tool",
      "arguments": {
        "message": "Hello from MCP!"
      }
    },
    "id": 2
  }' | jq .
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": "Echo: Hello from MCP!"
  },
  "id": 2
}
```

### Health Check

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"ping","id":3}' | jq .
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": "pong",
  "id": 3
}
```

## Code Structure

- `src/lib.rs` - Main MCP handler implementation
- `spin.toml` - Spin application configuration
- `Cargo.toml` - Rust dependencies

## Extending the Example

To add new tools:

1. Add the tool definition in the `Request::ToolsList` match arm
2. Add the tool implementation in the `Request::ToolsCall` match arm

Example:
```rust
Request::ToolsList => {
    Response::ToolsList(vec![
        // ... existing tools
        Tool {
            name: "greet".to_string(),
            description: "Greets a person by name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                },
                "required": ["name"]
            }).to_string(),
        }
    ])
}

Request::ToolsCall(params) => {
    match params.name.as_str() {
        // ... existing tools
        "greet" => {
            let args: serde_json::Value = serde_json::from_str(&params.arguments)
                .unwrap_or(json!({}));
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            Response::ToolsCall(ToolResult::Text(
                format!("Hello, {}! Welcome to MCP.", name)
            ))
        }
        _ => // ... error handling
    }
}
```