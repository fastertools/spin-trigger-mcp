# Spin MCP Trigger Plugin

A Spin trigger plugin that enables WebAssembly components to serve as Model Context Protocol (MCP) servers.

## Features

- Full MCP protocol support (tools, resources, prompts)
- JSON-RPC 2.0 over HTTP transport
- Easy deployment of MCP servers as Spin applications
- Compatible with Claude Desktop and other MCP clients

## Quick Start

See the [Getting Started Guide](docs/getting-started.md) for detailed installation and usage instructions.

## Installation

From source:

```bash
git clone https://github.com/fastertools/spin-trigger-mcp.git
cd spin-trigger-mcp
make
```

This will build and install both the trigger plugin and the MCP Rust template.

## Usage

### 1. Create an MCP Component

Create a new Spin application with an MCP trigger:

```toml
# spin.toml
spin_manifest_version = 2

[application]
name = "my-mcp-server"
version = "0.1.0"

[[trigger.mcp]]
component = "weather-tool"
route = "/weather"

[component.weather-tool]
source = "target/wasm32-wasip1/release/my_mcp_server.wasm"
```

### 2. Implement the MCP Interface

Your component must implement the MCP interface defined in `spin-mcp.wit`:

```rust
use spin_mcp_sdk::*;

struct Component;

impl Guest for Component {
    fn handle_request(request: Request) -> Response {
        match request {
            Request::ToolsList => {
                Response::ToolsList(vec![
                    Tool {
                        name: "get_weather".to_string(),
                        description: "Get weather for a location".to_string(),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "location": {
                                    "type": "string",
                                    "description": "City name"
                                }
                            },
                            "required": ["location"]
                        }).to_string(),
                    }
                ])
            }
            Request::ToolsCall(params) => {
                // Handle tool execution
                Response::ToolsCall(ToolResult::Json(
                    serde_json::json!({
                        "temperature": "72°F",
                        "conditions": "Sunny"
                    }).to_string()
                ))
            }
            _ => Response::Error(Error {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            })
        }
    }
}
```

### 3. Run Your MCP Server

```bash
spin build
spin up
```

Your MCP server is now running and can be accessed by MCP clients.

## MCP Client Configuration

### Claude Desktop

Add to your Claude Desktop config:

```json
{
  "mcpServers": {
    "weather": {
      "command": "curl",
      "args": ["-X", "POST", "http://localhost:3000/weather", "-H", "Content-Type: application/json", "-d", "@-"]
    }
  }
}
```

### Direct API Usage

```bash
# List available tools
curl -X POST http://localhost:3000/weather \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'

# Call a tool
curl -X POST http://localhost:3000/weather \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_weather",
      "arguments": {"location": "San Francisco"}
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
- `mcp-weather-tool/` - Weather information tool (work in progress)

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/spin-trigger-mcp
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