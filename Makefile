.PHONY: all build install clean cli mcp gpui

all: build

build: cli mcp gpui

cli:
	cd inkwell-cli && cargo build --release

mcp:
	cd inkwell-mcp && cargo build --release

gpui:
	cd inkwell-gpui && cargo build --release

install: build
	mkdir -p ~/.local/bin
	cp inkwell-cli/target/release/inkwell ~/.local/bin/
	cp inkwell-mcp/target/release/inkwell-mcp ~/.local/bin/
	cp inkwell-gpui/target/release/inkwell-gpui ~/.local/bin/
	@echo "Installe dans ~/.local/bin/"
	@echo "  inkwell       — CLI"
	@echo "  inkwell-mcp   — MCP Server"
	@echo "  inkwell-gpui  — Desktop App"

clean:
	cd inkwell-cli && cargo clean
	cd inkwell-mcp && cargo clean
	cd inkwell-gpui && cargo clean
