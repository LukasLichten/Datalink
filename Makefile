.phony: all build build-full debug-build bridge-debug bridge-release lib-test lib-example clean help

all: build-full

build: bridge-release
	cargo build --release

build-full: bridge-debug bridge-release
	cargo build -F "include-debug" --release

debug-build: bridge-debug
	cargo build

bridge-debug:
	cd datalink-shm-bridge && cargo build

bridge-release:
	cd datalink-shm-bridge && cargo build --release

clean: 
	cargo clean

lib-test:
	cd datalink-bridge-config && cargo test --all-features

lib-example:
	cd datalink-bridge-config && cargo run --all-features --example create_acc_config

help:
	@echo "Builds and test the shm-bridge"
	@echo "make:                Builds Full"
	@echo "make build:          Builds in release mode"
	@echo "make build-full:     Builds in release mode with --debug flag available"
	@echo "make debug-build:    Builds in debug mode"
	@echo "make bridge-debug:   Builds on the datalink-shm-bridge.exe in debug"
	@echo "make bridge-release: Builds on the datalink-shm-bridge.exe in debug"
	@echo "make lib-test:       Tests the config library"
	@echo "make lib-example:    Runs the ACC config generation example"
	@echo "make clean:          Cleans out build artifacts"
	@echo "make help:           This Printout"
