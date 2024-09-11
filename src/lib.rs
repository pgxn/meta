#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
PGXN Metadata validation.

This crate uses JSON Schema to validate and inspect the PGXN `META.json`
files. It supports both the [v1] and [v2] specs.

[v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
[v2]: https://github.com/pgxn/rfcs/pull/3

*/

pub mod meta;
mod util; // private utilities
pub mod valid;

#[cfg(test)]
mod tests;
