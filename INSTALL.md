# MTPScript Installation Guide

## Quick Install

For a quick installation, run:

```bash
curl -o- https://raw.githubusercontent.com/mytechpassport/mtpscript/main/install.sh | bash
```

This will install MTPScript to `~/.mtpscript` and add it to your PATH.

## Manual Installation

### Prerequisites

- **Rust**: Install Rust toolchain from [rustup.rs](https://rustup.rs)
- **GCC**: Ensure GCC is installed (e.g., `apt install build-essential` on Ubuntu)
- **Git**: For cloning the repository

### Clone the Repository

```bash
git clone https://github.com/mytechpassport/mtpscript.git
cd mtpscript
```

## Build Rust Components

```bash
cargo build --release
```

This builds the Rust crates (`mtpscript` CLI and `mtpscript-core` library).

## Build C Runtime Components

```bash
make setup
make compile
```

This compiles the C runtime executables (`mtpjs_repl` and `mtpsc`).

## Run Tests

```bash
cargo test
make test
```

## Usage

- **Compile**: `mtpc input.mtp output.msqs`
- **Run**: `mtpjs snapshot.msqs`
- **Compile and Run**: `mtpscript execute input.mtp`
- **CLI Help**: `mtpscript --help`
- **REPL**: `mtpjs` (interactive mode)