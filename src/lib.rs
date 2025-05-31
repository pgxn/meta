#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
PGXN `META.json` validation and management.

This crate uses JSON Schema to validate and inspect PGXN `META.json`
files. It supports both the [v1] and [v2] specs.

[v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
[v2]: https://github.com/pgxn/rfcs/pull/3

*/

pub mod dist;
pub mod error;
pub mod release;
mod util; // private utilities
pub mod valid;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
