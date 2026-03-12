.PHONY: build-core build-server build all clean check fmt test publish-core publish-wasm publish-server publish

CARGO_BIN := $(shell rustup which cargo 2>/dev/null | xargs dirname 2>/dev/null)
export PATH := $(CARGO_BIN):$(PATH)

all: build

build-core:
	cd core && wasm-pack build --target nodejs

build-server: build-core
	cd server && npm install && npm run build

build: build-server

check:
	cd core && cargo check && cargo clippy -- -D warnings

test:
	cd core && cargo test

fmt:
	cd core && cargo fmt --check
	cd server && npx prettier --check "src/**/*.ts"

fmt-fix:
	cd core && cargo fmt
	cd server && npx prettier --write "src/**/*.ts"

lint:
	cd server && npx eslint src

clean:
	rm -rf core/pkg core/target server/dist server/node_modules

# ─── Publish ──────────────────────────────────────────────────────────────────

## Publish the Rust crate to crates.io
publish-core:
	cd core && cargo publish

## Build the WASM package and publish it to npm
publish-wasm: build-core
	cd core/pkg && npm publish --access public

## Build the MCP server and publish it to npm (requires markdown-btree-core on npm)
publish-server: build-server
	cd server && npm publish --access public

## Full publish: Rust crate → npm WASM package → npm server (run in order)
publish: publish-core publish-wasm publish-server
