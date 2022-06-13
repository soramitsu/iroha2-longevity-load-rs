## About

This is a script that generates empty transactions, given a `TPS` rate. It reports status to a given `address`.
It was mainly created to test longevity stand for Iroha2.

## Usage

### Prerequisites

1. Rust 1.60.0 ([Installation guide](https://www.rust-lang.org/tools/install))
2. Edit `config.json` depending on your setup
3. Network should be started and have committed a genesis block
4. Have the Iroha repository locally.

### Installing

Clone *this* repository

```bash
git clone https://github.com/soramitsu/iroha2-longevity-load-rs
```


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
