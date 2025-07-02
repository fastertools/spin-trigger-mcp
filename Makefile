# Makefile to build and run MCP plugin for Spin framework.

.PHONY: all
all: build_plugin install_plugin install_template

.PHONY: lint
lint:
	@echo "Running linting check..."
	cargo clippy --all --all-targets -- -D warnings
	cargo fmt --all -- --check

.PHONY: lint-rust-examples
lint-rust-examples:	
		@echo "Running linting check on example..." \
		&& cargo clippy --manifest-path "examples/mcp-weather-tool/Cargo.toml" -- -D warnings \
		&& cargo fmt --manifest-path "examples/mcp-weather-tool/Cargo.toml" -- --check \

.PHONY: lint-all
lint-all: lint lint-rust-examples

.PHONY: build_plugin
build_plugin:
	@echo "Building MCP Plugin..."
	cargo build --release

.PHONY: install_plugin
install_plugin:
	@echo "Installing MCP Plugin in Spin..."
	spin plugins update && spin plugins upgrade pluginify -y
	spin pluginify --install

.PHONY: install_template
install_template:
	@echo "Installing MCP Rust template..."
	spin templates install --dir templates --update
	
.PHONY: clean
clean:
	@echo "Cleaning up..."
	cargo clean
	cargo clean --manifest-path ./examples/mcp-weather-tool/Cargo.toml
	rm -f trigger-mcp-*.tar.gz
	rm -f trigger-mcp.json
	spin plugin uninstall trigger-mcp