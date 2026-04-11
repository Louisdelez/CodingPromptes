.PHONY: all build install clean cli mcp gpui devtools-mcp

all: build

build: cli mcp gpui devtools-mcp

cli:
	cd inkwell-cli && cargo build --release

mcp:
	cd inkwell-mcp && cargo build --release

gpui:
	cd inkwell-gpui && cargo build --release

devtools-mcp:
	cd inkwell-devtools-mcp && cargo build --release

install: build
	mkdir -p ~/.local/bin
	cp inkwell-cli/target/release/inkwell ~/.local/bin/
	cp inkwell-mcp/target/release/inkwell-mcp ~/.local/bin/
	cp inkwell-gpui/target/release/inkwell-gpui ~/.local/bin/
	cp inkwell-devtools-mcp/target/release/inkwell-devtools-mcp ~/.local/bin/
	@echo "Installe dans ~/.local/bin/"
	@echo "  inkwell              — CLI"
	@echo "  inkwell-mcp          — MCP Server"
	@echo "  inkwell-gpui         — Desktop App"
	@echo "  inkwell-devtools-mcp — DevTools MCP"

clean:
	cd inkwell-cli && cargo clean
	cd inkwell-mcp && cargo clean
	cd inkwell-gpui && cargo clean
	cd inkwell-devtools-mcp && cargo clean
