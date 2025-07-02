# Contributing to Spin MCP Trigger Plugin

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/spin-trigger-mcp`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Commit with descriptive message
7. Push to your fork: `git push origin my-feature`
8. Open a Pull Request

## Development Setup

```bash
# Install Rust toolchain
rustup target add wasm32-wasip1

# Install Spin CLI
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

# Build the plugin
make build

# Run tests
make test
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add tests for new functionality
- Update documentation as needed

## Testing

- Unit tests: Test individual functions
- Integration tests: Test full MCP request/response flow
- Example validation: Ensure examples build and run

## Documentation

- Update README.md for user-facing changes
- Update docs/ for technical changes
- Add inline documentation for public APIs
- Include examples for new features

## Pull Request Process

1. Update CHANGELOG.md with your changes
2. Ensure all tests pass
3. Update documentation
4. Request review from maintainers

## Release Process

Maintainers will:
1. Update version numbers
2. Create GitHub release
3. Publish to plugin registry

## Questions?

Open an issue for discussion!