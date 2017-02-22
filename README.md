# GELF for Rust (`gelf`)

[![Crates.io](https://img.shields.io/crates/d/gelf.svg?style=flat-square)](https://crates.io/crates/gelf)
[![Build Status](https://img.shields.io/travis/bzikarsky/gelf-rust.svg?style=flat-square)](https://travis-ci.org/bzikarsky/gelf-rust)

A GELF implementation for Rust ([Documentation](https://docs.rs/gelf))

*TODO: What's GELF?*
- *Link Graylog*
- *Link GELF spec*

*TODO: What's this library?*

*TODO: GELF example usecases*

## Features

*TODO: ::log-integration, list of backends, conversion of error levels*

## Install

To make use of GELF for Rust, simply add it as a dependency in your `Cargo.toml`. Check for the latest
version at [cargo.io](https://cargo.io/gelf):

```toml
[dependencies]
gelf = "a.b.c"
```

If you installed [`cargo-edit`](https://github.com/killercup/cargo-edit) you can easily add the latest
version by running:

```
cargo add gelf
```

Finally add the crate to your application:

```rust
extern crate gelf;
```

## Examples & use
Two introductory examples (for both standalone and `log`-integrated uses) can be found 
[in the crate's documentation](https://docs.rs/gelf/).

Additional examples covering different backends and other advanced uses can be found in [`/examples`](examples).
Every one of those can be run with ´cargo´, e.g.:

```
cargo run --example simple_udp
```

## Documentation

The documentation is available at https://docs.rs/gelf and will get built automatically for every crate version.

## License

GELF for rust (`gelf`) is licensed under the [MIT-License](https://github.com/bzikarsky/gelf-rust/blob/master/LICENSE).

## Contact & Contributing

Contributions are very welcome. I will lay out a guide for contributions in a `CONTRIBUTING.md`. Until then 
you are invited to PR/issue as you like :-)

If you have any questions, feel free to contact me by [mail](mailto:benjamin@zikarsky.de), 
[Twitter](https://twitter.com/bzikarsky) or on IRC. I'll usually idle as `bzikarsky` on
[freenode](https://freenode.net) in #graylog. 

*TODO: CONTRIBUTING.md*





