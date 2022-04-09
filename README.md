# mx25r-rs
Driver for the MX25R chip using the [embedded-hal](https://github.com/rust-embedded/embedded-hal), note that this crate is still a **work in progress**.

## TODO
* Complete example to test most features
* Use the embedded-hal crate instead of the embassy fork.
* Add async suport
* Implement [embedded_storage nor_flash trait](https://docs.rs/embedded-storage/latest/embedded_storage/nor_flash/index.html)

## Usage
You can see an example of the usage for the `nRF52840-DK` in the [examples directory](./examples/nrf52840-dk). For now only a blocking API is available.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
