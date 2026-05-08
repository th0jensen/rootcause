# rootcause

This is my submission for solving "Root cause analysis in a dynamic IP network"

## Building

### Requirements
- Stable Rust toolchain from [rustup](https://rustup.rs/)

```sh
git clone https://github.com/th0jensen/rootcause
cd rootcause
cargo build --release
```

## Usage

The program expects input files matching the spec. There's a collection of example files in the `/examples` directory.

```sh
./target/release/rootcause -i ./examples/input1.json -e ./examples/events1.json
```

To view usage in the terminal:

```sh
./target/release/rootcause --help
```

## Project Structure

```
rootcause/
├── examples/                  # Example files
└── src/
    ├── main.rs                # CLI and API orchestration
    ├── types.rs               # Struct definitions for deserialization
    ├── network.rs             # Network and Event processing
    └── analysis.rs            # Score aggregation and analysis
```

## Libraries used

- `serde` and `serde_json`: JSON deserialization
- `clap`: CLI argument parsing
- `anyhow`: Unified error handling
- `jiff`: Timestamp parsing and formatting
