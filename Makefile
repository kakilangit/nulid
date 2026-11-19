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
# The `sqlx-postgres-uuid` and `sqlx-postgres-bytea` features are mutually
# exclusive (they pick the PostgreSQL storage mapping for NULIDs), so the
# lint/test targets run the crate twice — once per mapping. `--all-features`
# would enable both and fail the exclusivity check; never use it here.
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

# The two PostgreSQL storage mappings, each combined with every other feature
# so a pass covers the whole crate. The mappings are mutually exclusive, and
# cargo unifies features across the workspace (nulid_derive's dev-dependency
# enables the uuid mapping), so:
# - the uuid pass runs the whole workspace,
# - the bytea pass is scoped to the nulid package (-p), where the unification
#   cannot pull the uuid mapping in.
UUID_FEATURES := sqlx-postgres-uuid,postgres-types,rkyv,jiff,macros,serde,derive,cli
BYTEA_FEATURES := sqlx-postgres-bytea,postgres-types,rkyv,jiff,macros,serde,derive,cli

.PHONY: install-rust
install-rust: ## Install Rust toolchain with required components
	rustup toolchain install $(RUST_VERSION)
	rustup component add rustfmt clippy --toolchain $(RUST_VERSION)

.PHONY: check
check: ## Run cargo check on all workspace members (both sqlx storage mappings)
	cargo +$(RUST_VERSION) check --workspace --features $(UUID_FEATURES)
	cargo +$(RUST_VERSION) check -p nulid --features $(BYTEA_FEATURES)

.PHONY: fmt
fmt: ## Format all code
	cargo +$(RUST_VERSION) fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo +$(RUST_VERSION) fmt --all --check

.PHONY: clippy
clippy: ## Run clippy lints (both sqlx storage mappings)
	cargo +$(RUST_VERSION) clippy --workspace --features $(UUID_FEATURES) --all-targets -- -D warnings
	cargo +$(RUST_VERSION) clippy -p nulid --features $(BYTEA_FEATURES) --all-targets -- -D warnings

.PHONY: test
test: ## Run all tests (both sqlx storage mappings)
	cargo +$(RUST_VERSION) test --workspace --features $(UUID_FEATURES)
	cargo +$(RUST_VERSION) test -p nulid --features $(BYTEA_FEATURES)

.PHONY: test-doc
test-doc: ## Run documentation tests
	cargo +$(RUST_VERSION) test --doc --features $(UUID_FEATURES)

.PHONY: bench
bench: ## Run benchmarks
	cargo +$(RUST_VERSION) bench --features $(UUID_FEATURES)

.PHONY: bench-test
bench-test: ## Run benchmarks in test mode (CI)
	cargo +$(RUST_VERSION) bench --features $(UUID_FEATURES) -- --test

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
	@echo "Running derive_features example..."
	cargo +$(RUST_VERSION) run --example derive_features --features derive,serde,uuid,sqlx,postgres-types
	@echo "Running serde_example..."
	cargo +$(RUST_VERSION) run --example serde_example --features serde
	@echo "Running uuid_conversion example..."
	cargo +$(RUST_VERSION) run --example uuid_conversion --features uuid
	@echo "Running rkyv_example..."
	cargo +$(RUST_VERSION) run --example rkyv_example --features rkyv
	@echo "Running postgres_types_example..."
	cargo +$(RUST_VERSION) run --example postgres_types_example --features postgres-types
	@echo "Running jiff_example..."
	cargo +$(RUST_VERSION) run --example jiff_example --features jiff
	@echo "Running wasm_example..."
	cargo +$(RUST_VERSION) run --example wasm_example --features wasm
	@echo "Running sqlx_sqlite example..."
	cargo +$(RUST_VERSION) run --example sqlx_sqlite --features sqlx

.PHONY: build
build: ## Build all workspace members (canonical sqlx mapping)
	cargo +$(RUST_VERSION) build --features $(UUID_FEATURES) --workspace

.PHONY: build-release
build-release: ## Build all workspace members in release mode (canonical sqlx mapping)
	cargo +$(RUST_VERSION) build --features $(UUID_FEATURES) --workspace --release

.PHONY: doc
doc: ## Generate documentation (canonical sqlx mapping)
	cargo +$(RUST_VERSION) doc --features $(UUID_FEATURES) --workspace --no-deps

.PHONY: doc-open
doc-open: ## Generate and open documentation (canonical sqlx mapping)
	cargo +$(RUST_VERSION) doc --features $(UUID_FEATURES) --workspace --no-deps --open

.PHONY: examples-sqlx
examples-sqlx: ## Run the sqlx PostgreSQL examples (needs DATABASE_URL and a database)
	@test -n "$$DATABASE_URL" || (echo "DATABASE_URL is not set"; exit 1)
	cargo +$(RUST_VERSION) run --example sqlx_postgres --features sqlx-postgres-uuid
	cargo +$(RUST_VERSION) run --example sqlx_postgres_bytea --features sqlx-postgres-bytea

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
	# The verification build uses the canonical UUID mapping: the two sqlx
	# storage mappings are mutually exclusive, so --all-features cannot compile.
	cargo publish --dry-run --features $(UUID_FEATURES)

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
	cargo publish --features $(UUID_FEATURES)

.PHONY: update-deps
update-deps: ## Update dependencies
	cargo update

.PHONY: integration
integration: ## Run database integration tests (requires Docker)
	./scripts/integration.sh --all

.PHONY: integration-postgres
integration-postgres: ## Run PostgreSQL integration test only
	./scripts/integration.sh --postgres

.PHONY: integration-mysql
integration-mysql: ## Run MySQL integration test only
	./scripts/integration.sh --mysql

.PHONY: integration-mariadb
integration-mariadb: ## Run MariaDB integration test only
	./scripts/integration.sh --mariadb

.PHONY: integration-cleanup
integration-cleanup: ## Cleanup integration test containers
	./scripts/integration.sh --cleanup

.PHONY: all
all: ci doc ## Run all checks and build documentation
