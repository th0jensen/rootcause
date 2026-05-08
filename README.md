# rootcause

This is my submission for solving "Root cause analysis in a dynamic IP network"

## Approach

Events are processed chronologically against a live topology model of the network.

`LINK_DOWN` events mutate the graph and score the affected link and node as candidates for the root cause.

`NODE_UNREACHABLE` events are checked against the current topology, and if the topology agrees then it's categorised as a downstream symptom of the root cause.

Candidates are ranked by accumulated score and split into most likely causes, less likely causes, and observed downstream symptoms.

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

Optionally, without building first:

```sh
cargo run -- -i ./examples/input1.json -e ./examples/events1.json
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

## Libraries/crates used

- `serde` and `serde_json`: JSON deserialization
- `clap`: CLI argument parsing
- `anyhow`: Unified error handling
- `jiff`: Timestamp parsing and formatting
