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

pack-tryhcs-app-wasm:
	cd tryhcs_app && wasm-pack build --release --target web

.PHONY: all build build-wasm