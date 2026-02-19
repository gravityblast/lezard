<p align="center">
  <img src="assets/lezard-logo.png" width="200" alt="Lezard logo" />
</p>

<h1 align="center">Lezard 🦎</h1>

<p align="center">
  A framework for building, testing, and deploying programs on the
  <a href="https://github.com/logos-blockchain/lssa">Logos Execution Zone (LEZ)</a>.
</p>

---

Lezard is a framework that provides helpers to spin up a local sequencer, deploy RISC Zero guest programs, send transactions, and verify on-chain state.

> **Early development** — Lezard is in its initial phase. For now, it's best to keep `lssa`, `lezard`, and your projects in the same parent directory. In the future Lezard will be a standalone framework that pulls its dependencies from crates.io / git.

## Prerequisites

- **Rust** — install the latest from [rustup.rs](https://rustup.rs)
- **Docker** — required by `cargo risczero build` to compile guest programs for RISC-V
- **RISC Zero** — follow the [installation guide](https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain)

## Getting started

```bash
# Create a working directory
mkdir lez-dev && cd lez-dev

# Clone the repos
git clone git@github.com:logos-blockchain/lssa.git
git clone git@github.com:gravityblast/lezard.git

# Scaffold a new project
./lezard/create-project.sh my-project
cd my-project

# Build the test guest programs
make build

# Run tests
make test
```

## Project structure

After running `create-project.sh`, your project looks like this:

```
my-project/
├── Cargo.toml                  # Depends on lezard framework
├── Makefile                    # build / test targets
├── programs/                   # Guest programs (compiled to RISC-V)
│   ├── Cargo.toml
│   └── src/bin/
│       ├── double.rs           # Example: doubles guest program
│       └── ...
└── tests/                      # Integration tests
    └── double_test.rs          # Deploys and test double.rs
```

**Guest programs** live in `programs/src/bin/`. Each `.rs` file becomes a separate RISC-V ELF binary.

**Tests** live in `tests/`. They use the `lezard` library to start a local sequencer, deploy programs, send transactions, and assert on-chain state.

## Contributing

Contributions are welcome! Feel free to open issues and pull requests.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

## License

MIT or Apache-2.0
