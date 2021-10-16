# async-sqlx-session

## DO NOT CONTRIBUTE TO THIS
This is a version of async-sqlx-session I'm having to vendor since tide's SessionStore hasn't been updated for async-session 3.0.0 yet. PRs for this library should be sent to https://github.com/http-rs/async-session/issues/24
and I will be replacing it ASAP with the version from crates.io.

## [sqlx](https://github.com/launchbadge/sqlx)-backed session store for [async-session](https://github.com/http-rs/async-session)

* [CI ![CI][ci-badge]][ci]
* [API Docs][docs] [![docs.rs docs][docs-badge]][docs]
* [Releases][releases] [![crates.io version][version-badge]][lib-rs]
* [Contributing][contributing]

[ci]: https://github.com/jbr/async-sqlx-session/actions?query=workflow%3ACI
[ci-badge]: https://github.com/jbr/async-sqlx-session/workflows/CI/badge.svg
[releases]: https://github.com/jbr/async-sqlx-session/releases
[docs]: https://docs.rs/async-sqlx-session
[contributing]: https://github.com/jbr/async-sqlx-session/blob/main/.github/CONTRIBUTING.md
[lib-rs]: https://lib.rs/async-sqlx-session
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[version-badge]: https://img.shields.io/crates/v/async-sqlx-session.svg?style=flat-square

## Installation
### sqlite: 

```toml
async-sqlx-session = { version = "0.4.0", features = ["sqlite"] }
```

### postgres: 

```toml
async-sqlx-session = { version = "0.4.0", features = ["pg"] }
```

### mysql: 

```toml
async-sqlx-session = { version = "0.4.0", features = ["mysql"] }
```

### Optional `async_std` feature

To use the `spawn_cleanup_task` function on the async-std runtime,
enable the `async_std` feature, which can be combined with any of the
above datastores.

```toml
async-sqlx-session = { version = "0.4.0", features = ["pg", "async_std"] }
```

## Cargo Features:

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
