# PCalc

A substrate runtime module for pi-calc-based smart contracts.

This modules [design principles and motivation](DesignAndMotivation.md) are also available.

This repository includes the runtime module itself in `runtime/src/pcalc.rs` as well as a basic node-template-based blockchain client that uses it.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

# Run

Start a development chain with:

```bash
cargo run -- --dev
```

Or see substrate.dev for lots more information about running substrate-based blockchains
