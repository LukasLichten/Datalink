.phony: all build build-full build-debug bridge-debug bridge-release lib-test lib-example clean help

all: build-debug

build: bridge-release
	cargo build --release

build-full: bridge-release bridge-console
	cargo build -F "include-debug" --release

build-debug: bridge-debug
	cargo build

bridge-debug:
	cd datalink-shm-bridge && cargo build -F "display-console"

bridge-release:
	cd datalink-shm-bridge && cargo build --release

bridge-console:
	cd datalink-shm-bridge && cargo build --profile release-include-console -F "display-console"

clean: 
	cargo clean

lib-test:
	cd datalink-bridge-config && cargo test --all-features

lib-example:
	cd datalink-bridge-config && cargo run --all-features --example create_acc_config

help:
	@echo "Builds and test the shm-bridge"
	@echo "make:                Builds in debug mode"
	@echo "make build:          Builds in release mode"
	@echo "make build-full:     Builds in release mode with --debug flag available"
	@echo "make build-debug:    Builds in debug mode"
	@echo "make bridge-debug:   Builds on the datalink-shm-bridge.exe in debug"
	@echo "make bridge-release: Builds on the datalink-shm-bridge.exe in release"
	@echo "make bridge-console: Builds on the datalink-shm-bridge.exe in release with debug console"
	@echo "make lib-test:       Tests the config library"
	@echo "make lib-example:    Runs the ACC config generation example"
	@echo "make clean:          Cleans out build artifacts"
	@echo "make help:           This Printout"
