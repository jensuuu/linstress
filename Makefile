BINARY := linstress

.DEFAULT_GOAL := help

.PHONY: help deps tools build run run-root dist check fmt lint clean \
        package deb rpm appimage packages icons

help: ## Show this help
	@grep -E '^[a-z-]+:.*##' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-10s %s\n", $$1, $$2}'

# --- Development --------------------------------------------------------------

deps: ## Install system libraries (apt, dnf or pacman)
	bash ./scripts/install-deps.sh

build: ## Build an optimized release binary
	cargo build --release

run: ## Run in development mode
	cargo run

run-root: build ## Run the release binary as root (needed for tuning)
	sudo -E ./target/release/$(BINARY)

dist: ## Build the maximum-optimization binary
	cargo build --profile dist

check: ## Fast compile check without building
	cargo check --all-targets

fmt: ## Format the code
	cargo fmt

lint: ## Run clippy, warnings are errors
	cargo fmt --check
	cargo lint

clean: ## Remove build output
	cargo clean
	rm -rf dist build

# --- Packaging (results go to dist/) -------------------------------------------

tools: ## Install cargo-deb and cargo-generate-rpm
	cargo install --locked cargo-deb cargo-generate-rpm

deb: build ## Create a .deb package
	mkdir -p dist
	cargo deb --no-build --output dist/

rpm: build ## Create an .rpm package
	mkdir -p dist
	cargo generate-rpm --output dist/

appimage: ## Create an AppImage
	bash ./scripts/build-appimage.sh

packages: deb rpm appimage ## Create .deb, .rpm and AppImage
	@ls -lh dist/

package: dist ## Create a plain dist/linux-stress-oc.tar.gz
	mkdir -p dist
	tar -czf dist/$(BINARY).tar.gz -C target/dist $(BINARY) -C ../.. README.md LICENSE
	@echo "Created dist/$(BINARY).tar.gz"

icons: ## Re-create the PNG icons from the SVG files
	bash ./scripts/generate-icons.sh
