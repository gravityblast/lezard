.PHONY: build test help

help:
	@echo "Available targets:"
	@echo "  make build  - Compile guest programs"
	@echo "  make test   - Run all tests"

build:
	rm -rf programs/.deps/nssa_core
	mkdir -p programs/.deps
	cp -r ../../lssa/nssa/core programs/.deps/nssa_core
	cd programs && cargo generate-lockfile
	CARGO_TARGET_DIR=target cargo risczero build --manifest-path programs/Cargo.toml
	rm -rf programs/.deps/nssa_core

test:
	RISC0_DEV_MODE=1 RUST_LOG="warn,lezard=info" cargo test --release
