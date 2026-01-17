.PHONY: help context context-program context-tools context-frontend format \
	format-rust format-frontend check-format test test-rust test-frontend \
	lint lint-rust lint-frontend type-check clean dev build install

help:
	@echo "Context generation commands:"
	@echo "  make context           - Generate full project context"
	@echo "  make context-program   - Generate on-chain program context"
	@echo "  make context-tools     - Generate tooling context (swap_ops/swap_sdk)"
	@echo "  make context-frontend  - Generate frontend context"
	@echo ""
	@echo "Code formatting commands:"
	@echo "  make format            - Format all code (Rust + frontend)"
	@echo "  make format-rust       - Format Rust code"
	@echo "  make format-frontend   - Format frontend code (if configured)"
	@echo "  make check-format      - Check formatting without modifying files"
	@echo ""
	@echo "Testing commands:"
	@echo "  make test              - Run all tests"
	@echo "  make test-rust          - Run Rust tests"
	@echo "  make test-frontend     - Run frontend tests"
	@echo ""
	@echo "Development commands:"
	@echo "  make dev               - Start frontend dev server"
	@echo "  make build             - Build Rust + frontend"
	@echo "  make install           - Install frontend dependencies"
	@echo ""
	@echo "Quality checks:"
	@echo "  make lint              - Run linters"
	@echo "  make type-check        - Run type checks"
	@echo ""
	@echo "Cleanup commands:"
	@echo "  make clean             - Remove generated context files and build artifacts"

# Context generation
context:
	@echo "Generating full project context..."
	@DATE=$$(date '+%Y-%m-%d_%H-%M-%S_%Z'); \
	OUTPUT_FILE="context-full-$${DATE}.xml"; \
	cp repomix.config.json repomix.config.json.bak && \
	jq ".output.filePath = \"$$OUTPUT_FILE\"" repomix.config.json > repomix.config.json.tmp && \
	mv repomix.config.json.tmp repomix.config.json && \
	(repomix --config repomix.config.json || (mv repomix.config.json.bak repomix.config.json && exit 1)) && \
	mv repomix.config.json.bak repomix.config.json && \
	rm -f repomix.config.json.tmp && \
	echo "✅ Context written to $$OUTPUT_FILE"

context-program:
	@echo "Generating program-focused context..."
	@DATE=$$(date '+%Y-%m-%d_%H-%M-%S_%Z'); \
	OUTPUT_FILE="context-program-$${DATE}.xml"; \
	cp repomix.config.json repomix.config.json.bak && \
	jq --arg file "$$OUTPUT_FILE" \
	  '.output.filePath = $$file | .include = ["solana-xmr-swap/programs/**", "solana-xmr-swap/test_vectors/**", "solana-xmr-swap/Anchor.toml", "solana-xmr-swap/Cargo.toml", "solana-xmr-swap/Cargo.lock", "solana-xmr-swap/SECURITY.md", "solana-xmr-swap/AUDIT_STATUS.md", "solana-xmr-swap/docs/**"]' \
	  repomix.config.json > repomix.config.json.tmp && \
	mv repomix.config.json.tmp repomix.config.json && \
	(repomix --config repomix.config.json || (mv repomix.config.json.bak repomix.config.json && exit 1)) && \
	mv repomix.config.json.bak repomix.config.json && \
	rm -f repomix.config.json.tmp && \
	echo "✅ Context written to $$OUTPUT_FILE"

context-tools:
	@echo "Generating tooling-focused context..."
	@DATE=$$(date '+%Y-%m-%d_%H-%M-%S_%Z'); \
	OUTPUT_FILE="context-tools-$${DATE}.xml"; \
	cp repomix.config.json repomix.config.json.bak && \
	jq --arg file "$$OUTPUT_FILE" \
	  '.output.filePath = $$file | .include = ["solana-xmr-swap/tools/**", "solana-xmr-swap/test_vectors/**", "solana-xmr-swap/docs/**", "solana-xmr-swap/README.md"]' \
	  repomix.config.json > repomix.config.json.tmp && \
	mv repomix.config.json.tmp repomix.config.json && \
	(repomix --config repomix.config.json || (mv repomix.config.json.bak repomix.config.json && exit 1)) && \
	mv repomix.config.json.bak repomix.config.json && \
	rm -f repomix.config.json.tmp && \
	echo "✅ Context written to $$OUTPUT_FILE"

context-frontend:
	@echo "Generating frontend-focused context..."
	@DATE=$$(date '+%Y-%m-%d_%H-%M-%S_%Z'); \
	OUTPUT_FILE="context-frontend-$${DATE}.xml"; \
	cp repomix.config.json repomix.config.json.bak && \
	jq --arg file "$$OUTPUT_FILE" \
	  '.output.filePath = $$file | .include = ["solana-xmr-swap/frontend/src/**", "solana-xmr-swap/frontend/index.html", "solana-xmr-swap/frontend/package.json", "solana-xmr-swap/frontend/vite.config.ts", "solana-xmr-swap/frontend/tsconfig*.json", "solana-xmr-swap/frontend/src/index.css", "solana-xmr-swap/README.md"]' \
	  repomix.config.json > repomix.config.json.tmp && \
	mv repomix.config.json.tmp repomix.config.json && \
	(repomix --config repomix.config.json || (mv repomix.config.json.bak repomix.config.json && exit 1)) && \
	mv repomix.config.json.bak repomix.config.json && \
	rm -f repomix.config.json.tmp && \
	echo "✅ Context written to $$OUTPUT_FILE"

# Formatting
format: format-rust format-frontend

format-rust:
	@echo "Formatting Rust code..."
	@cd solana-xmr-swap && cargo fmt

format-frontend:
	@echo "Formatting frontend code..."
	@cd solana-xmr-swap/frontend && npm run lint -- --fix || echo "No frontend formatter configured"

check-format:
	@echo "Checking formatting..."
	@cd solana-xmr-swap && cargo fmt -- --check
	@cd solana-xmr-swap/frontend && npm run lint || echo "No frontend linter configured"

# Testing
test: test-rust test-frontend

test-rust:
	@echo "Running Rust tests..."
	@cd solana-xmr-swap && cargo test

test-frontend:
	@echo "Running frontend tests..."
	@cd solana-xmr-swap/frontend && npm test

# Development
dev:
	@echo "Starting frontend dev server..."
	@cd solana-xmr-swap/frontend && npm run dev

build:
	@echo "Building Rust + frontend..."
	@cd solana-xmr-swap && cargo build
	@cd solana-xmr-swap/frontend && npm run build

install:
	@echo "Installing frontend dependencies..."
	@cd solana-xmr-swap/frontend && npm install

# Quality checks
lint: lint-rust lint-frontend

lint-rust:
	@echo "Running Rust clippy..."
	@cd solana-xmr-swap && cargo clippy --all-targets --all-features

lint-frontend:
	@echo "Running frontend lint..."
	@cd solana-xmr-swap/frontend && npm run lint

type-check:
	@echo "Running type checks..."
	@cd solana-xmr-swap && cargo check
	@cd solana-xmr-swap/frontend && npx tsc -b --pretty false

# Cleanup
clean:
	@echo "Cleaning generated files..."
	@rm -f context*.xml
	@rm -f context-*.xml
	@rm -f repomix.config.json.bak
	@rm -f repomix.config.json.tmp
	@rm -rf solana-xmr-swap/frontend/dist
	@rm -rf solana-xmr-swap/target
	@echo "✅ Cleanup complete"
