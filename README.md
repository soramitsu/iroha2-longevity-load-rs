# Iroha 2 Longevity Load

## About

This is a script that generates transactions according to given operations and given a `TPS` (transactions per second) rate. It reports the status to a given `address` in the `daemon` mode or the standard output in the `oneshot` mode.
It was mainly created to test longevity stand for Iroha 2.

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

### Building

- Build from sources
    ```bash
    cargo build
    ```

- Or build using Docker
    ```bash
    docker build .
    ```

### Running

In the project folder:

Get help how to use the CLI app:
```bash
./iroha2-longevity-load-rs --help
```

#### Daemon mode

Run as a server in the background and it executes given operation
```bash
./iroha2-longevity-load-rs daemon --operation RegisterAccount
```

To run multiple operations simultaneously, you should use several `--operation` flags
```bash
./iroha2-longevity-load-rs daemon \
    --operation RegisterAccount \
    --operation RegisterDomain
```

To get the status (port `8084` by default), use CURL
```bash
curl 127.0.0.1:8084
```

#### One-shot mode

Run a single operation in the foreground and wait for the result that will be printed to stdout
```bash
./iroha2-longevity-load-rs oneshot --operation RegisterAccount
```

### Operations
Here is a list of operations you can use

- `RegisterAccount` - it registers a new account.
- `RegisterDomain` - it registers a new domain.
- `RegisterAssetQuantity` - it registers a new quantity asset with a random mintable mode.
- `RegisterAssetBigQuantity` - it registers a new big quantity asset with a random mintable mode.
- `RegisterAssetFixed` - it registers a new fixed asset with a random mintable mode.
- `RegisterAssetStore` - it registers a new store asset with metadata containing a random value.
- `TransferAsset` - it registers two accounts with assets and transfers one (TODO: make this randomly generated) value between them.
- `MintAsset` - it registers a new asset and a new account that owns this asset, and then mints one (TODO: make this randomly generated) asset.
