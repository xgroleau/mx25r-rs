# mx25r-rs
Platform-agnostic Rust driver for the macronix MX25R NOR flash using the [embedded-hal](https://github.com/rust-embedded/embedded-hal).


This driver implements all the commands available to the MX25R chip series, but qspi is not supported yet.
Note that the drivers are low level to allow the user to write custom implementation for its needs.

## Usage
You can see an example of the usage for the `nRF52840-DK` in the [nrf52840 directory](./nrf52840). For now only a blocking API is available.

### Nix
A [nix flake](https://nixos.wiki/wiki/Flakes) is available to ease development and dependencies for the examples.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
