## About

This is a testing script that generates empty transactions, with given `TPS` and reports status at a given `address`.
It was mainly created to test longevity stand for Iroha2.

## Usage

### Prerequisites

1. Rust 1.57.0 ([Installation guide](https://www.rust-lang.org/tools/install))
2. Edit `config.json` depending on your setup 
3. Network should be started and have committed a genesis block

### Running

In the project folder:

Get help how to set arguments (TPS, Address):
```
cargo run -- --help
```

Run with default arguments
```
cargo run
```

Get the status (port `8084` by default)
```
curl 127.0.0.1:8084
```
