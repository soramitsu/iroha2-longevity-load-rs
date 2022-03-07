## About

This is a testing script that generates empty transactions, with given `TPS` and reports status at a given `address`.
It was mainly created to test longevity stand for Iroha2.

## Usage

### Prerequisites

1. Rust 1.57.0 ([Installation guide](https://www.rust-lang.org/tools/install))
2. Edit `config.json` depending on your setup
3. Network should be started and have committed a genesis block
4. Have the Iroha repository locally.

### Installing

Clone *this* repository

```bash
git clone https://github.com/soramitsu/iroha2-longevity-load-rs
```

Clone Iroha

```bash
git clone https://github.com/hyperledger/iroha.git
```

and check it out to the current release branch: `iroha2` (or `iroha2-dev`):

```bash
cd iroha
git checkout iroha2
```

return to the root directory

```bash
cd ../iroha2-longevity-load-rs
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
