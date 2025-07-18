.PHONY: help prereqs setup dev dev-fast server client wasm wasm-debug bevy-client bevy-client-quick bevy-client-release bevy-dev test lint format clean check

# Default target
help: ## Show this help
	@echo "Boid Wars Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  make \033[36m%-12s\033[0m %s\n", $$1, $$2}'

prereqs: ## Check all prerequisites
	@./scripts/check-prereqs.sh

setup: ## Initial project setup
	@echo "🚀 Setting up Boid Wars..."
	@./scripts/check-prereqs.sh
	@if [ ! -f .env ]; then cp .env.example .env; echo "✅ Created .env file"; fi
	@if [ ! -f $$HOME/.boid-wars/certs/localhost.pem ]; then ./scripts/setup-certs.sh; fi
	@cargo build --all
	@./scripts/build-wasm.sh
	@echo "✅ Setup complete! Run 'make dev' to start developing"


server: ## Run only the game server
	@./scripts/run-server.sh


wasm: ## Build WASM module (release)
	@./scripts/build-wasm.sh

wasm-debug: ## Build WASM module (debug, faster)
	@./scripts/build-wasm-debug.sh

test: ## Run all tests
	@echo "🧪 Running Rust tests..."
	@cargo test --all

lint: ## Run all linters
	@echo "🔍 Running Rust linter..."
	@cargo clippy --all -- -D warnings

format: ## Format all code
	@echo "✨ Formatting Rust code..."
	@cargo fmt --all

check: ## Run all checks (fmt, clippy, tests)
	@cargo fmt --all -- --check
	@cargo clippy --all-features -- -D warnings
	@cargo test --all

bevy-client: ## Build Bevy WASM client (development)
	@echo "🎮 Building Bevy WASM client (development)..."
	@cd bevy-client && ./build-wasm.sh

bevy-client-quick: ## Quick Bevy WASM build (no clean, debug mode)
	@echo "⚡ Quick Bevy WASM build..."
	@cd bevy-client && ./build-quick.sh

bevy-client-release: ## Build optimized Bevy WASM client (release)
	@echo "🚀 Building Bevy WASM client (release)..."
	@cd bevy-client && ./build-release.sh

dev: ## Run server and Bevy WASM client for development
	@echo "🎮 Starting Bevy WASM development mode..."
	@echo "📦 Building Bevy client if needed..."
	@cd bevy-client && ./build-quick.sh > /dev/null 2>&1
	@echo "🚀 Starting server and Bevy client..."
	@echo "📝 Server logs and client server logs will be interleaved. Press Ctrl+C to stop all."
	@echo "🌐 Client will be available at http://localhost:8081"
	@echo ""
	@set -m; \
	trap 'echo "🛑 Stopping servers..."; pkill -f boid-wars-server; pkill -f "python.*http.server"; pkill -f "python.*SimpleHTTPServer"; exit 0' INT; \
	cargo run --bin boid-wars-server & \
	SERVER_PID=$$!; \
	(cd bevy-client && (command -v python3 >/dev/null && python3 -m http.server 8081 || python -m SimpleHTTPServer 8081)) & \
	CLIENT_PID=$$!; \
	wait

clean: ## Clean all build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf lightyear-wasm/pkg
	@rm -rf bevy-client/pkg
	@echo "✅ Clean complete"

build: ## Build all components for production
	@echo "🏗️  Building for production..."
	@cargo build --release --all
	@./scripts/build-wasm.sh
	@echo "✅ Production build complete"