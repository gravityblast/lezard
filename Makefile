.PHONY: build-programs help

help:
	@echo "Available targets:"
	@echo "  make build-programs  - Compile guest programs for RISC-V (risc0)"

build-programs:
	rm -rf programs/.deps/nssa_core
	mkdir -p programs/.deps
	cp -r ../lssa/nssa/core programs/.deps/nssa_core
	cd programs && cargo generate-lockfile
	CARGO_TARGET_DIR=target cargo risczero build --manifest-path programs/Cargo.toml
	rm -rf programs/.deps/nssa_core

test:
	# RISC0_DEV_MODE=1 RUST_LOG=info cargo test --release --test double_test
	RISC0_DEV_MODE=1 RUST_LOG=info cargo test --release
