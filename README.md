# racegate

## Setup

### Prerequisites

You need nightly Rust and `espflash`. To install both of them, just use the
commands below:

```shell
rustup install nightly-2023-04-04
rustup component add rust-src --toolchain nightly-2023-04-04
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

### Debugging

#### Built in JTAG interface

Not all ESP32-C3 have JTAG interface. Example:

| Model                                                     | JTAG |
|-----------------------------------------------------------|------|
| [ESP32-C3-DevKit-RUST-1](https://mou.sr/40F3w6d)          | Yes  |
| [M5Stamp C3U](https://docs.m5stack.com/en/core/stamp_c3u) | Yes  |
| [M5Stamp C3](https://docs.m5stack.com/en/core/stamp_c3u)  | No   |


#### External JTAG interface

You need an *ESP-Prog Board*.

Reference: https://espressif-docs.readthedocs-hosted.com/projects/espressif-esp-iot-solution/en/latest/hw-reference/ESP-Prog_guide.html

Connect the JTAG interface to ESP32C3:

| JTAG | ESP32C3 |
|------|---------|
| TMS  | GPIO4   |
| TDI  | GPIO5   |
| TCK  | GPIO6   |
| TDO  | GPIO7   |

**TODO**