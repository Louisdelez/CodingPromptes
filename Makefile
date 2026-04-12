.PHONY: all build install uninstall clean test clippy run cli mcp gpui devtools-mcp

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
	mkdir -p ~/.local/bin ~/.local/share/inkwell-ide
	cp inkwell-cli/target/release/inkwell ~/.local/bin/
	cp inkwell-mcp/target/release/inkwell-mcp ~/.local/bin/
	cp inkwell-gpui/target/release/inkwell-gpui ~/.local/bin/
	cp inkwell-devtools-mcp/target/release/inkwell-devtools-mcp ~/.local/bin/
	@echo "Installe dans ~/.local/bin/"
	@echo "  inkwell              — CLI (18 commandes)"
	@echo "  inkwell-mcp          — MCP Server (10 outils)"
	@echo "  inkwell-gpui         — Desktop App (GPUI, 60fps)"
	@echo "  inkwell-devtools-mcp — DevTools MCP (45+ outils)"

uninstall:
	rm -f ~/.local/bin/inkwell
	rm -f ~/.local/bin/inkwell-mcp
	rm -f ~/.local/bin/inkwell-gpui
	rm -f ~/.local/bin/inkwell-devtools-mcp
	@echo "Desinstalle de ~/.local/bin/"

test:
	cd inkwell-core && cargo test
	@echo "Tests inkwell-core OK"

clippy:
	cd inkwell-core && cargo clippy -- -W warnings
	cd inkwell-cli && cargo clippy -- -W warnings
	@echo "Clippy OK"

run: gpui
	RUST_LOG=info ./inkwell-gpui/target/release/inkwell-gpui

clean:
	cd inkwell-cli && cargo clean
	cd inkwell-mcp && cargo clean
	cd inkwell-gpui && cargo clean
	cd inkwell-devtools-mcp && cargo clean
