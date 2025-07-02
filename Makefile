.PHONY: build install test clean release

# Build the plugin
build:
	cargo build --release

# Install the plugin locally
install: build
	spin pluginify --install

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -f trigger-mcp.json
	rm -f *.tar.gz

# Build and package for release
release: build
	spin pluginify

# Build example
example: build
	cd examples/mcp-weather-tool && spin build

# Run example
run-example: example
	cd examples/mcp-weather-tool && spin up