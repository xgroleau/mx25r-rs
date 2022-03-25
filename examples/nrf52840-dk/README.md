# `app-template`

> Quickly set up a [`probe-run`] + [`defmt`] + [`flip-link`] embedded project

[`probe-run`]: https://crates.io/crates/probe-run
[`defmt`]: https://github.com/knurling-rs/defmt
[`flip-link`]: https://github.com/knurling-rs/flip-link

## Dependencies

#### 1. `flip-link`:

```console
$ cargo install flip-link
```

#### 2. `probe-run`:

``` console
$ # make sure to install v0.2.0 or later
$ cargo install probe-run
```

#### 3. [`cargo-generate`]:

``` console
$ cargo install cargo-generate
```

[`cargo-generate`]: https://crates.io/crates/cargo-generate

> *Note:* You can also just clone this repository instead of using `cargo-generate`, but this involves additional manual adjustments.

## Setup

### Softdevice
You need to flash the softdevice when the device is erased.
``` sh
probe-rs-cli download --format hex softdevices/s140_nrf52_7.2.0_softdevice.hex  --chip nRF52840_xxAA --chip-erase
```

## Running tests

The template comes configured for running unit tests and integration tests on the target.

Unit tests reside in the library crate and can test private API; the initial set of unit tests are in `src/lib.rs`.
`cargo test --lib` will run those unit tests.

``` console
$ cargo test --lib
(1/1) running `it_works`...
└─ app::unit_tests::__defmt_test_entry @ src/lib.rs:33
all tests passed!
└─ app::unit_tests::__defmt_test_entry @ src/lib.rs:28
```

Integration tests reside in the `tests` directory; the initial set of integration tests are in `tests/integration.rs`.
`cargo test --test integration` will run those integration tests.
Note that the argument of the `--test` flag must match the name of the test file in the `tests` directory.

``` console
$ cargo test --test integration
(1/1) running `it_works`...
└─ integration::tests::__defmt_test_entry @ tests/integration.rs:13
all tests passed!
└─ integration::tests::__defmt_test_entry @ tests/integration.rs:8
```

Note that to add a new test file to the `tests` directory you also need to add a new `[[test]]` section to `Cargo.toml`.

## Trying out the git version of defmt

This template is configured to use the latest crates.io release (the "stable" release) of the `defmt` framework.
To use the git version (the "development" version) of `defmt` follow these steps:

1. Install the *git* version of `probe-run`

``` console
$ cargo install --git https://github.com/knurling-rs/probe-run --branch main
```

2. Check which defmt version `probe-run` supports

``` console
$ probe-run --version
0.2.0 (aa585f2 2021-02-22)
supported defmt version: 60c6447f8ecbc4ff023378ba6905bcd0de1e679f
```

In the example output, the supported version is `60c6447f8ecbc4ff023378ba6905bcd0de1e679f`

3. Switch defmt dependencies to git: uncomment the last part of the root `Cargo.toml` and enter the hash reported by `probe-run --version`:

``` diff
-# [patch.crates-io]
-# defmt = { git = "https://github.com/knurling-rs/defmt", rev = "use defmt version reported by `probe-run --version`" }
-# defmt-rtt = { git = "https://github.com/knurling-rs/defmt", rev = "use defmt version reported by `probe-run --version`" }
-# defmt-test = { git = "https://github.com/knurling-rs/defmt", rev = "use defmt version reported by `probe-run --version`" }
-# panic-probe = { git = "https://github.com/knurling-rs/defmt", rev = "use defmt version reported by `probe-run --version`" }
+[patch.crates-io]
+defmt = { git = "https://github.com/knurling-rs/defmt", rev = "60c6447f8ecbc4ff023378ba6905bcd0de1e679f" }
+defmt-rtt = { git = "https://github.com/knurling-rs/defmt", rev = "60c6447f8ecbc4ff023378ba6905bcd0de1e679f" }
+defmt-test = { git = "https://github.com/knurling-rs/defmt", rev = "60c6447f8ecbc4ff023378ba6905bcd0de1e679f" }
+panic-probe = { git = "https://github.com/knurling-rs/defmt", rev = "60c6447f8ecbc4ff023378ba6905bcd0de1e679f" }
```

You are now using the git version of `defmt`!

**NOTE** there may have been breaking changes between the crates.io version and the git version; you'll need to fix those in the source code.

## Support

`app-template` is part of the [Knurling] project, [Ferrous Systems]' effort at
improving tooling used to develop for embedded systems.

If you think that our work is useful, consider sponsoring it via [GitHub
Sponsors].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Knurling]: https://knurling.ferrous-systems.com
[Ferrous Systems]: https://ferrous-systems.com/
[GitHub Sponsors]: https://github.com/sponsors/knurling-rs
