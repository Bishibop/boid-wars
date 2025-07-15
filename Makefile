.PHONY: help dev build test clean setup

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

setup: ## Initial project setup
	cargo install cargo-watch bacon wasm-pack
	npm install
	cd deploy && mkcert localhost 127.0.0.1 ::1

dev: ## Run all development servers
	npm run dev

build: ## Build all components for production
	npm run build

test: ## Run all tests
	cargo test --all
	cd client && npm test

clean: ## Clean all build artifacts
	cargo clean
	rm -rf client/node_modules client/dist
	rm -rf lightyear-wasm/pkg

check: ## Run all checks (fmt, clippy, tests)
	cargo fmt --all -- --check
	cargo clippy --all-features -- -D warnings
	cargo test --all

server: ## Run only the game server
	cd server && cargo run

client: ## Run only the client dev server
	cd client && npm run dev

wasm: ## Build WASM module
	cd lightyear-wasm && wasm-pack build --target web --out-dir ../client/src/wasm