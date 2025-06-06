# Makefile for building Cargo projects and tryhcs_app for wasm

BINDINGS_DIR=tryhcs_app/bindings
PKG_DIR=tryhcs_app/pkg
BARREL_FILE=tryhcs_app/bindings/index.d.ts
BARREL_PATH=$(PKG_DIR)/$(BARREL_FILE)

# Default target
all: build build-wasm

# Build all Cargo projects in the workspace
build:
	cargo build --workspace

check-tryhcs-app-native:
	cargo check -p tryhcs_app

check-tryhcs-app-wasm:
	cargo check -p tryhcs_app --target wasm32-unknown-unknown

check: check-tryhcs-app-native check-tryhcs-app-wasm

build-tryhcs-app-native:
	cargo fmt -p tryhcs_app && cargo build -p tryhcs_app


test-tryhcs-app-native:
	cargo fmt -p tryhcs_app && cargo test -p tryhcs_app

build-tryhcs-app-ts:
	cargo fmt -p tryhcs_app && cargo test export_bindings -p tryhcs_app

build-tryhcs-app-wasm:
	cargo fmt -p tryhcs_app && cargo build -p tryhcs_app --target wasm32-unknown-unknown

build-tryhcs-app: build-tryhcs-app-native build-tryhcs-app-wasm

internal-tryhcs-app-generate-barrel:
	@echo "Generating barrel file..."
	@echo "// Auto-generated barrel file" > $(BARREL_FILE)
	@for file in $(BINDINGS_DIR)/*.ts; do \
		filename=$$(basename $$file .ts); \
		echo "export * from './$$filename';" >> $(BARREL_FILE); \
	done

internal-tryhcs-app-move-barrel:
	@echo "Moving barrel file to $(PKG_DIR)..."
	@mv tryhcs_app/bindings tryhcs_app/pkg

internal-tryhcs-app-wasm-fix-pkg:
	sed -i '6i\"bindings/",' tryhcs_app/pkg/package.json
	sed -i '3i\export type * from "./bindings/index.d.ts";' tryhcs_app/pkg/tryhcs_app.d.ts
	
pack-tryhcs-app-wasm:
	cd tryhcs_app && rm -rf pkg && wasm-pack build --release --target web

push-tryhcs-app-wasm:
	yalc push tryhcs_app/pkg

tryhcs-app-dev: build-tryhcs-app-ts pack-tryhcs-app-wasm internal-tryhcs-app-generate-barrel internal-tryhcs-app-move-barrel internal-tryhcs-app-wasm-fix-pkg push-tryhcs-app-wasm

platform-run:
	cargo fmt -p tryhcs-platform && cargo run -p tryhcs-platform

.PHONY: all build build-wasm