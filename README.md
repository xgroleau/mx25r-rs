# mx25r-rs
Driver for the MX25R chip using the [embedded-hal](https://github.com/rust-embedded/embedded-hal), note that this crate is still a **work in progress**. It only suports the MX25R6435F for now.

## TODO
* Complete sync low level driver
* Add support all mx25r family
  * Other macronix chips? (L, S, U)
* Add typestate programming
* Use embedded hal 1.0 alpha instead of 0.2.x
* Add async suport

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
