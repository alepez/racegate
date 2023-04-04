# racegate

## Setup

### Prerequisites

You need nightly Rust and `espflash`. To install both of them, just use the
commands below:

```shell
rustup install nightly-2023-01-22
rustup component add rust-src --toolchain nightly-2023-01-22
cargo install cargo-espflash ldproxy
```

### Configuration

You can configure some options in `cfg.toml`. This options will be embedded in
the resulting binary. This feature is provided
by [toml-cfg](https://crates.io/crates/toml-cfg)

The file `cfg.toml` is not provided. But you can copy `cfg.toml.example`
to `cfg.toml` and edit it.

## Development

### Flash

To build, flash and see logs, you can use `cargo espflash`:

```shell
cargo espflash --speed 1500000 --release --monitor /dev/ttyACM0
```