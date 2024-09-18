.phony: all build debug-build clean help

all: build

build: 
	cd datalink-shm-bridge && cargo build --release
	cargo build --release

debug-build:
	cd datalink-shm-bridge && cargo build
	cargo build

clean: 
	cargo clean

help:
	@echo "Builds and test the shm-bridge"
	@echo "make:             Builds"
	@echo "make build:       Builds in release mode"
	@echo "make debug-build: Builds in debug mode"
	@echo "make clean:       Cleans out build artifacts"
	@echo "make help:        This Printout"
