# ==============================================================================
# Makefile for nulid workspace
# ==============================================================================
#
# This Makefile provides convenient commands for development, testing, and
# publishing the nulid workspace. It uses the same commands as the GitHub
# Actions CI/CD workflows to ensure consistency between local development
# and continuous integration.
#
# The Rust version is automatically extracted from the workspace Cargo.toml
# to ensure alignment with the project's rust-version requirement.
#
# Usage:
#   make help          - Show all available targets
#   make ci            - Run all CI checks (fmt-check, clippy, test, etc.)
#   make pre-commit    - Run pre-commit checks (fmt, clippy, test)
#   make publish       - Publish all crates to crates.io
#
# ==============================================================================

.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Rust version - automatically extracted from workspace Cargo.toml
RUST_VERSION := $(shell grep 'rust-version = ' Cargo.toml | head -1 | sed 's/.*rust-version = "\(.*\)"/\1/')

.PHONY: install-rust
install-rust: ## Install Rust toolchain with required components
	rustup toolchain install $(RUST_VERSION)
	rustup component add rustfmt clippy --toolchain $(RUST_VERSION)

.PHONY: check
check: ## Run cargo check on all workspace members
	cargo +$(RUST_VERSION) check --all-features --workspace

.PHONY: fmt
fmt: ## Format all code
	cargo +$(RUST_VERSION) fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo +$(RUST_VERSION) fmt --all --check

.PHONY: clippy
clippy: ## Run clippy lints
	cargo +$(RUST_VERSION) clippy --all-features --all-targets -- -D warnings

.PHONY: test
test: ## Run all tests
	cargo +$(RUST_VERSION) test --all-features

.PHONY: test-doc
test-doc: ## Run documentation tests
	cargo +$(RUST_VERSION) test --doc --all-features

.PHONY: bench
bench: ## Run benchmarks
	cargo +$(RUST_VERSION) bench --all-features

.PHONY: bench-test
bench-test: ## Run benchmarks in test mode (CI)
	cargo +$(RUST_VERSION) bench --all-features -- --test

.PHONY: examples
examples: ## Run all examples
	@echo "Running basic example..."
	cargo +$(RUST_VERSION) run --example basic
	@echo "Running monotonic example..."
	cargo +$(RUST_VERSION) run --example monotonic
	@echo "Running derive_wrapper example..."
	cargo +$(RUST_VERSION) run --example derive_wrapper --features derive
	@echo "Running macros example..."
	cargo +$(RUST_VERSION) run --example macros --features macros
	@echo "Running combined_features example..."
	cargo +$(RUST_VERSION) run --example combined_features --features derive,macros
	@echo "Running serde_example..."
	cargo +$(RUST_VERSION) run --example serde_example --features serde
	@echo "Running uuid_conversion example..."
	cargo +$(RUST_VERSION) run --example uuid_conversion --features uuid
	@echo "Running rkyv_example..."
	cargo +$(RUST_VERSION) run --example rkyv_example --features rkyv
	@echo "Running postgres_types_example..."
	cargo +$(RUST_VERSION) run --example postgres_types_example --features postgres-types

.PHONY: build
build: ## Build all workspace members
	cargo +$(RUST_VERSION) build --all-features --workspace

.PHONY: build-release
build-release: ## Build all workspace members in release mode
	cargo +$(RUST_VERSION) build --all-features --workspace --release

.PHONY: doc
doc: ## Generate documentation
	cargo +$(RUST_VERSION) doc --all-features --workspace --no-deps

.PHONY: doc-open
doc-open: ## Generate and open documentation
	cargo +$(RUST_VERSION) doc --all-features --workspace --no-deps --open

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: ci
ci: fmt-check clippy test test-doc bench-test examples ## Run all CI checks

.PHONY: pre-commit
pre-commit: fmt clippy test ## Run pre-commit checks

.PHONY: verify-version
verify-version: ## Verify version consistency across workspace
	@echo "Checking workspace configuration..."
	@VERSION=$$(grep '^\s*version = ' Cargo.toml | head -1 | sed 's/.*version = "\(.*\)"/\1/'); \
	echo "  Workspace version: $$VERSION"; \
	EDITION=$$(grep '^\s*edition = ' Cargo.toml | head -1 | sed 's/.*edition = "\(.*\)"/\1/'); \
	echo "  Workspace edition: $$EDITION"; \
	RUST_VER=$$(grep '^\s*rust-version = ' Cargo.toml | head -1 | sed 's/.*rust-version = "\(.*\)"/\1/'); \
	echo "  Workspace rust-version: $$RUST_VER"; \
	echo ""; \
	echo "Checking sub-crates..."; \
	DERIVE_CHECK=$$(grep 'version.workspace = true' nulid_derive/Cargo.toml > /dev/null && echo "workspace ($$VERSION)" || grep '^version = ' nulid_derive/Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	echo "  nulid_derive version: $$DERIVE_CHECK"; \
	MACROS_CHECK=$$(grep 'version.workspace = true' nulid_macros/Cargo.toml > /dev/null && echo "workspace ($$VERSION)" || grep '^version = ' nulid_macros/Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
	echo "  nulid_macros version: $$MACROS_CHECK"

.PHONY: publish-dry-run
publish-dry-run: ## Dry run of publishing to crates.io
	@echo "Dry run: Publishing nulid_derive..."
	cd nulid_derive && cargo publish --dry-run
	@echo "Dry run: Publishing nulid_macros..."
	cd nulid_macros && cargo publish --dry-run
	@echo "Dry run: Publishing nulid..."
	cargo publish --dry-run --all-features

.PHONY: publish
publish: ## Publish all crates to crates.io (requires CARGO_REGISTRY_TOKEN)
	@echo "Publishing nulid_derive..."
	cd nulid_derive && cargo publish
	@echo "Waiting for crates.io propagation..."
	sleep 30
	@echo "Publishing nulid_macros..."
	cd nulid_macros && cargo publish
	@echo "Waiting for crates.io propagation..."
	sleep 30
	@echo "Publishing nulid..."
	cargo publish --all-features

.PHONY: update-deps
update-deps: ## Update dependencies
	cargo update

.PHONY: audit
audit: ## Run security audit
	cargo audit

.PHONY: bloat
bloat: ## Analyze binary bloat
	cargo bloat --release --all-features

.PHONY: tree
tree: ## Show dependency tree
	cargo tree --all-features

.PHONY: outdated
outdated: ## Check for outdated dependencies
	cargo outdated

.PHONY: all
all: ci doc ## Run all checks and build documentation
