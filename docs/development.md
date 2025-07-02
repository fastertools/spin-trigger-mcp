# MCP Trigger Plugin Development Guide

## Overview

The MCP (Model Context Protocol) trigger plugin enables Spin applications to serve as MCP servers, exposing tools, resources, and prompts to AI assistants like Claude.

## Architecture

The plugin follows Spin's trigger architecture:

1. **Trigger Implementation** (`src/lib.rs`): Implements the `Trigger` trait
2. **HTTP Transport**: Handles JSON-RPC 2.0 over HTTP
3. **WIT Bindings**: Uses `wasmtime::component::bindgen!` to generate bindings
4. **Component Interface**: Components implement the MCP interface defined in `spin-mcp.wit`

## Building the Plugin

### Prerequisites

- Rust toolchain with `wasm32-wasip1` target
- Spin CLI v3.0+

### Build Steps

```bash
# Build the plugin
cargo build --release

# Package for distribution
spin pluginify

# Install locally for testing
spin pluginify --install
```

## Creating MCP Components

### 1. Generate from Template

```bash
spin new -t mcp-rust my-mcp-server
```

### 2. Implement the MCP Interface

Components must implement `handle_request` to process MCP requests:

```rust
impl exports::spin::mcp_trigger::mcp_types::Guest for Component {
    fn handle_request(request: Request) -> Response {
        match request {
            Request::ToolsList => {
                // Return available tools
            }
            Request::ToolsCall(params) => {
                // Execute tool
            }
            // Handle other request types...
        }
    }
}
```

### 3. Define Tools

Tools are functions that AI models can call:

```rust
Tool {
    name: "get_data".to_string(),
    description: "Retrieve data from database".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "SQL query to execute"
            }
        },
        "required": ["query"]
    }).to_string(),
}
```

## Testing

### Unit Tests

Test your component logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tools_list() {
        let response = Component::handle_request(Request::ToolsList);
        // Assert expected tools
    }
}
```

### Integration Tests

Test with the running server:

```bash
# Start the server
spin up

# Test tools/list
curl -X POST http://localhost:3000/my-tool \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'
```

## Debugging

Enable debug logging:

```bash
RUST_LOG=debug spin up
```

Common issues:

1. **Component not found**: Check `spin.toml` trigger configuration
2. **Method not found**: Verify request handling in `handle_request`
3. **Invalid JSON Schema**: Validate with JSON Schema tools

## Best Practices

1. **Input Validation**: Always validate tool arguments
2. **Error Handling**: Return proper MCP errors, not panics
3. **JSON Schemas**: Use detailed schemas for better AI understanding
4. **Documentation**: Provide clear descriptions for tools and parameters
5. **Security**: Never expose sensitive operations without safeguards

## Advanced Features

### Resources

Expose data that AI can read:

```rust
Request::ResourcesList => {
    Response::ResourcesList(vec![
        Resource {
            uri: "config://app".to_string(),
            name: "Application Config".to_string(),
            description: Some("Current app configuration".to_string()),
            mime_type: Some("application/json".to_string()),
        }
    ])
}
```

### Prompts

Provide prompt templates:

```rust
Request::PromptsList => {
    Response::PromptsList(vec![
        Prompt {
            name: "analyze_data".to_string(),
            description: Some("Analyze dataset".to_string()),
            arguments: vec![
                PromptArgument {
                    name: "dataset".to_string(),
                    description: Some("Dataset to analyze".to_string()),
                    required: true,
                }
            ],
        }
    ])
}
```

## Performance Considerations

1. **Async Operations**: Use async for I/O operations
2. **Resource Management**: Clean up resources in tool implementations
3. **Caching**: Cache expensive computations when appropriate
4. **Timeouts**: Implement reasonable timeouts for external calls

## Security

1. **Authentication**: Implement auth at the HTTP layer if needed
2. **Input Sanitization**: Validate and sanitize all inputs
3. **Rate Limiting**: Consider rate limiting for resource-intensive tools
4. **Secrets Management**: Use Spin's secret store for sensitive data

## Publishing

1. Update version in `Cargo.toml` and `spin-pluginify.toml`
2. Build and test thoroughly
3. Create GitHub release with binaries
4. Submit to Spin plugin registry

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.