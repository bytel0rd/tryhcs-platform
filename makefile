# Makefile for building Cargo projects and tryhcs_app for wasm

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


structure-tryhcs-app-wasm-pkg:
	sed -i '6i\"bindings/",' tryhcs_app/pkg/package.json
	sed -i '3i\export type * from "./bindings/index.d.ts";' tryhcs_app/pkg/tryhcs_app.d.ts
	
pack-tryhcs-app-wasm:
	cd tryhcs_app && wasm-pack build --release --target web

push-tryhcs-app-wasm:
	yalc push tryhcs_app/pkg

tryhcs-app-dev: pack-tryhcs-app-wasm structure-tryhcs-app-wasm-pkg push-tryhcs-app-wasm

platform-run:
	cargo fmt -p tryhcs-platform && cargo run -p tryhcs-platform

.PHONY: all build build-wasm