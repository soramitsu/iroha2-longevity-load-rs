## About

This is a script that generates transactions according to given operations and given a `TPS` (transactions per second) rate. It reports the status to a given `address` in the `daemon` mode or the standard output in the `oneshot` mode.
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
```
./iroha2-longevity-load-rs --help
```

#### Daemon mode

Run as a server in the background and it executes given operation
```
./iroha2-longevity-load-rs daemon --operation RegisterAccount
```

To run multiple operations simultaneously, you should use several `--operation` flags
```
./iroha2-longevity-load-rs daemon \
    --operation RegisterAccount \
    --operation RegisterDomain
```

To get the status (port `8084` by default)
```
curl 127.0.0.1:8084
```

#### One-shot mode

Run a single operation in the foreground and wait for the result
```
./iroha2-longevity-load-rs oneshot --operation RegisterAccount
```