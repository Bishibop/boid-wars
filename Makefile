.PHONY: help prereqs setup dev server client wasm test lint format clean check

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
	@npm install
	@cd client && npm install
	@cargo build --all
	@./scripts/build-wasm.sh
	@echo "✅ Setup complete! Run 'make dev' to start developing"

dev: wasm ## Run both server and client (hot reload)
	@echo "🚀 Starting development servers..."
	@echo "📝 Logs will be interleaved. Press Ctrl+C to stop all."
	@echo ""
	@trap 'kill %1 %2' INT; \
	./scripts/run-server.sh & \
	(cd client && npm run dev) & \
	wait

server: ## Run only the game server
	@./scripts/run-server.sh

client: ## Run only the client dev server
	@cd client && npm run dev

wasm: ## Build WASM module
	@./scripts/build-wasm.sh

test: ## Run all tests
	@echo "🧪 Running Rust tests..."
	@cargo test --all
	@echo ""
	@echo "🧪 Running client tests..."
	@cd client && npm test

lint: ## Run all linters
	@echo "🔍 Running Rust linter..."
	@cargo clippy --all -- -D warnings
	@echo ""
	@echo "🔍 Running TypeScript linter..."
	@cd client && npm run lint

format: ## Format all code
	@echo "✨ Formatting Rust code..."
	@cargo fmt --all
	@echo "✨ Formatting TypeScript code..."
	@cd client && npm run format

check: ## Run all checks (fmt, clippy, tests)
	@cargo fmt --all -- --check
	@cargo clippy --all-features -- -D warnings
	@cargo test --all
	@cd client && npm run lint
	@cd client && npm run type-check

clean: ## Clean all build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf client/dist client/src/wasm
	@rm -rf lightyear-wasm/pkg
	@echo "✅ Clean complete"

build: ## Build all components for production
	@echo "🏗️  Building for production..."
	@cargo build --release --all
	@./scripts/build-wasm.sh
	@cd client && npm run build
	@echo "✅ Production build complete"