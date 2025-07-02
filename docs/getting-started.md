# Getting Started with Spin MCP Trigger

This guide will walk you through creating and running your first MCP (Model Context Protocol) server using the Spin MCP trigger plugin.

## Prerequisites

1. [Spin](https://developer.fermyon.com/spin/v2/install) v2.0 or later
2. [Rust](https://www.rust-lang.org/tools/install) with the `wasm32-wasip1` target:
   ```bash
   rustup target add wasm32-wasip1
   ```

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/fastertools/spin-trigger-mcp.git
   cd spin-trigger-mcp
   ```

2. Build and install the trigger plugin:
   ```bash
   make
   ```

   This will:
   - Build the trigger plugin
   - Package it for your platform
   - Install it as a Spin plugin
   - Install the MCP Rust template

3. Verify the installation:
   ```bash
   spin plugins list
   ```
   You should see `trigger-mcp` in the list.

## Creating Your First MCP Server

1. Create a new MCP component:
   ```bash
   spin new -t mcp-rust my-mcp-server
   ```

   You'll be prompted for:
   - **Description**: A description of your MCP server (default: "An MCP server component")
   - **HTTP route**: The route path for your MCP server (default: "/mcp")

   To accept all defaults, use:
   ```bash
   spin new -t mcp-rust my-mcp-server --accept-defaults
   ```

2. Navigate to your project:
   ```bash
   cd my-mcp-server
   ```

3. Build the component:
   ```bash
   spin build
   ```

4. Run the MCP server:
   ```bash
   spin up
   ```

   Your MCP server is now running at `http://localhost:3000/mcp` (or your custom route).

## Testing Your MCP Server

The default template includes an example tool. You can test it using curl:

1. **List available tools:**
   ```bash
   curl -X POST http://localhost:3000/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' | jq .
   ```

2. **Call a tool:**
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

3. **Other endpoints:**
   ```bash
   # Ping
   curl -X POST http://localhost:3000/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"ping","id":3}' | jq .

   # List resources
   curl -X POST http://localhost:3000/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"resources/list","id":4}' | jq .

   # List prompts
   curl -X POST http://localhost:3000/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"prompts/list","id":5}' | jq .
   ```

## Customizing Your MCP Server

Edit `src/lib.rs` to add your own tools, resources, and prompts. The template provides a basic structure:

```rust
use spin_mcp_sdk::{mcp_component, Request, Response, Tool, ToolResult, Error};
use serde_json::json;

#[mcp_component]
fn handle_request(request: Request) -> Response {
    match request {
        Request::ToolsList => {
            Response::ToolsList(vec![
                Tool {
                    name: "your_tool".to_string(),
                    description: "Description of your tool".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "param": {
                                "type": "string",
                                "description": "Parameter description"
                            }
                        },
                        "required": ["param"]
                    }).to_string(),
                }
            ])
        }
        Request::ToolsCall(params) => {
            match params.name.as_str() {
                "your_tool" => {
                    // Implement your tool logic here
                    Response::ToolsCall(ToolResult::Text(
                        "Tool response".to_string()
                    ))
                }
                _ => Response::ToolsCall(ToolResult::Error(Error {
                    code: -32602,
                    message: format!("Unknown tool: {}", params.name),
                    data: None,
                }))
            }
        }
        // ... other request types
    }
}
```

## Configuration

### Application Configuration (spin.toml)

- **Route**: Change the MCP endpoint route in the `[[trigger.mcp]]` section
- **Outbound hosts**: Allow HTTP requests by uncommenting and configuring `allowed_outbound_hosts`

### Running on Different Ports

By default, the MCP trigger listens on port 3000. To use a different port:

```bash
spin up --listen 0.0.0.0:8080
```

## Next Steps

- Check out the [examples](../examples/) directory for more complex MCP servers
- Read the [MCP specification](https://modelcontextprotocol.io/) to learn about all available features
- Join the [Spin Discord](https://discord.gg/fermyon) for help and discussions

## Troubleshooting

### Empty reply from server

If you get `curl: (52) Empty reply from server`, check:
1. The server logs for any errors
2. Your JSON syntax - consider saving complex requests to a file and using `curl -d @file.json`
3. That you're using the correct route (default is `/mcp`)

### Component not found

If you see `no such component` errors:
1. Ensure your `spin.toml` component name matches your project name
2. Try rebuilding with `spin build`
3. Make sure you have the latest trigger plugin installed