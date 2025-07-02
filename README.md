# Spin MCP Trigger Plugin

A Spin trigger plugin that enables WebAssembly components to serve as Model Context Protocol (MCP) servers.

## Features

- Full MCP protocol support (tools, resources, prompts)
- JSON-RPC 2.0 over HTTP transport
- Easy deployment of MCP servers as Spin applications
- Compatible with Claude Desktop and other MCP clients

## Quick Start

### 1. Install the Plugin and Template

From source:

```bash
git clone https://github.com/fastertools/spin-trigger-mcp.git
cd spin-trigger-mcp
make
```

This will:
- Build and install the `trigger-mcp` plugin
- Install the `mcp-rust` template for creating new MCP components

Verify installation:
```bash
spin plugins list
# Should show: trigger-mcp

spin templates list
# Should show: mcp-rust (MCP server component)
```

### 2. Create a New MCP Tool

Use the template to create a new MCP server:

```bash
spin new -t mcp-rust my-mcp-server
cd my-mcp-server
```

This creates a project structure with:
- `spin.toml` - Spin application manifest with MCP trigger configuration
- `src/lib.rs` - Rust code with the MCP component implementation
- `Cargo.toml` - Rust dependencies including the spin-mcp-sdk

### 3. Implement Your MCP Tools

Your component uses the `mcp_component` macro to implement the MCP interface:

```rust
use spin_mcp_sdk::{mcp_component, Request, Response, Tool, ToolResult, Error};
use serde_json::json;

#[mcp_component]
fn handle_request(request: Request) -> Response {
    match request {
        Request::ToolsList => {
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
            Response::ResourcesList(vec![])
        }
        
        Request::PromptsList => {
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
```

### 4. Build and Run Your MCP Server

```bash
spin build
spin up
```

Your MCP server is now running at `http://localhost:3000/mcp` and can be accessed by MCP clients.

## MCP Client Configuration

### Claude Desktop

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "my-mcp-server": {
      "command": "curl",
      "args": ["-X", "POST", "http://localhost:3000/mcp", "-H", "Content-Type: application/json", "-d", "@-"]
    }
  }
}
```

### Direct API Usage

```bash
# List available tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "example_tool",
      "arguments": {"message": "Hello, MCP!"}
    },
    "id": 2
  }'
```

## Supported MCP Features

- **Tools**: Expose functions that AI models can call
- **Resources**: Share data like files or database schemas
- **Prompts**: Provide prompt templates for AI interactions
- **Full JSON-RPC 2.0**: Complete protocol implementation

## Examples

See the `examples/` directory for complete MCP server implementations:

- `demo-mcp/` - Simple echo tool demonstrating basic MCP functionality

To run the demo example:
```bash
cd examples/demo-mcp
spin build
spin up
```

Then test it:
```bash
# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'

# Call the echo tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "example_tool",
      "arguments": {
        "message": "Hello, MCP!"
      }
    },
    "id": 2
  }'
```

## Creating Custom Tools

To add more tools to your MCP server, modify the `src/lib.rs` file:

1. Add your tool to the `ToolsList` response
2. Handle the tool call in the `ToolsCall` match
3. Return appropriate responses using `ToolResult::Text` or `ToolResult::Error`

Example of adding a calculator tool:

```rust
Request::ToolsList => {
    Response::ToolsList(vec![
        Tool {
            name: "add".to_string(),
            description: "Add two numbers".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["a", "b"]
            }).to_string(),
        }
    ])
}

Request::ToolsCall(params) => {
    match params.name.as_str() {
        "add" => {
            let args: serde_json::Value = serde_json::from_str(&params.arguments)?;
            let a = args["a"].as_f64().unwrap_or(0.0);
            let b = args["b"].as_f64().unwrap_or(0.0);
            Response::ToolsCall(ToolResult::Text(
                format!("{} + {} = {}", a, b, a + b)
            ))
        }
        _ => // ... handle unknown tool
    }
}
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/fastertools/spin-trigger-mcp
cd spin-trigger-mcp

# Build the plugin
cargo build --release

# Package for distribution
spin pluginify
```

### Running Tests

```bash
cargo test
```

## License

Apache-2.0