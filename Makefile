.phony: all build debug-build lib-test lib-example clean help

all: build

build: 
	cd datalink-shm-bridge && cargo build --release
	cargo build --release

debug-build:
	cd datalink-shm-bridge && cargo build
	cargo build

clean: 
	cargo clean

lib-test:
	cd datalink-memmap-config && cargo test --all-features

lib-example:
	cd datalink-memmap-config && cargo run --all-features --example create_acc_config

help:
	@echo "Builds and test the shm-bridge"
	@echo "make:             Builds"
	@echo "make build:       Builds in release mode"
	@echo "make debug-build: Builds in debug mode"
	@echo "make lib-test:    Tests the config library"
	@echo "make lib-example: Runs the ACC config generation example"
	@echo "make clean:       Cleans out build artifacts"
	@echo "make help:        This Printout"
