#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/*!
PGXN Metadata validation.

This crate uses JSON Schema to validate PGXN Meta Spec `META.json` files.
It supports both the [v1] and [v2] specs.

# Example

``` rust
use std::{path::PathBuf, error::Error};
use serde_json::json;
use pgxn_meta::*;

let meta = json!({
  "name": "pair",
  "abstract": "A key/value pair data type",
  "version": "0.1.8",
  "maintainers": [{ "name": "theory", "email": "theory@pgxn.org" }],
  "license": "PostgreSQL",
  "contents": {
    "extensions": {
      "pair": {
        "sql": "sql/pair.sql",
        "control": "pair.control"
      }
    }
  },
  "meta-spec": { "version": "2.0.0" }
});

let mut validator = Validator::new();
assert!(validator.validate(&meta).is_ok());
# Ok::<(), Box<dyn Error>>(())
```

[v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
[v2]: https://rfcs.pgxn.org/0003-meta-spec-v2.html

*/

mod util;
mod valid;
pub use valid::{ValidationError, Validator};
mod meta;
// pub use meta::*;

#[cfg(test)]
mod tests;
