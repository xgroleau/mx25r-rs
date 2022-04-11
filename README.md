# mx25r-rs
Platform-agnostic Rust driver for the macronix MX25R NOR flash using the [embedded-hal](https://github.com/rust-embedded/embedded-hal), note that this crate is still a **work in progress**.

This driver implements all the commands available to the MX25R chip series, but quad spi is not supported yet.

Note that the drivers are low level to allow the user to write custom implementation for it's needs.

## TODO
* Complete example to test most features
* Use the embedded-hal crate instead of the embassy fork.
* Add async suport
* Add qspi support

## Usage
You can see an example of the usage for the `nRF52840-DK` in the [examples directory](./examples/nrf52840-dk). For now only a blocking API is available.

Things to consider when using the driver crate

* Enable write before erasing sector/block/chip or writing data to the memory
  * Even when using the `embedded_storage` `NorFlash` trait
* Poll the wip bit before read/write/erase operation, if not the request will be ignored
* `write_security_register` is not a reversable operation, make sure to read the datasheet

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
