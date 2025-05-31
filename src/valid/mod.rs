/*!
PGXN Distribution Metadata validation.

This module uses JSON Schema to validate PGXN `META.json` files.
It supports both the [v1] and [v2] specs.

# Example

``` rust
# use pgxn_meta::error::Error;
use serde_json::json;
use pgxn_meta::valid::*;

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
# Ok::<(), Error>(())
```

  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3

*/
use crate::{error::Error, util};
use boon::{Compiler, Schemas};
use log::debug;
use serde_json::Value;

/// Export compiler publicly only for tests.
#[cfg(test)]
pub mod compiler;

#[cfg(not(test))]
mod compiler;

/// PGXN Meta validator.
pub struct Validator {
    compiler: Compiler,
    schemas: Schemas,
}

/// The base URL for all JSON schemas.
const SCHEMA_BASE: &str = "https://pgxn.org/meta/v";

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    /// Validator constructor.
    ///
    /// new creates and returns a new Validator with the schemas loaded from
    /// `dir`.
    ///
    /// ``` rust
    /// use pgxn_meta::valid::*;
    /// let validator = Validator::new();
    /// ```
    pub fn new() -> Validator {
        Validator {
            compiler: compiler::new(),
            schemas: Schemas::new(),
        }
    }

    /// Validates PGXN distribution metadata.
    ///
    /// Load a distribution `META.json` file into a serde_json::value::Value
    /// and pass it for validation. Returns the Meta spec version (1 or 2) on
    /// success and a validation error on failure.
    ///
    /// See the [module docs](crate::valid) for an example.
    pub fn validate(&mut self, meta: &Value) -> Result<u8, Error> {
        self.validate_schema(meta, "distribution.schema.json")
    }

    /// Validates PGXN release distribution metadata.
    ///
    /// On release, PGXN adds release metadata to the distribution `META.json`
    /// and publishes it separately so that clients can find and validate a
    /// release. A v1 `META.json` file include the user who published the
    /// release, the release timestamp, and a sha1 checksums for the
    /// distribution file. [RFC 5] defines the structure of v2 release
    /// metadata as a [JSON Web Signature], which includes an encoded payload
    /// value which must be separately validated by [`Self::validate_payload`].
    ///
    ///
    /// This method validates the structure of such a release `META.json`
    /// file. Load one up into a [serde_json::value::Value] and pass it for
    /// validation. Returns the Meta spec version (1 or 2) on success and a
    /// validation error on failure.
    ///
    /// [JSON Serialization]: https://datatracker.ietf.org/doc/html/rfc7515#section-7.2
    /// [RFC 5]: https://github.com/pgxn/rfcs/pull/5
    /// [JSON Web Signature]: https://datatracker.ietf.org/doc/html/rfc7515
    pub fn validate_release(&mut self, meta: &Value) -> Result<u8, Error> {
        self.validate_schema(meta, "release.schema.json")
    }

    /// Validate PGXN release JWS payload.
    ///
    /// The JSON Web Signature [JSON Serialization] object validated by
    /// [`Self::validate_release`] includes a Base 64 URL-encoded payload,
    /// which contains the validated PGXN release metadata. Once decoded, use
    /// this method to validate it.
    ///
    /// The payload includes the user who published the release, the release
    /// timestamp, and checksums for the distribution file, as defined by [RFC
    /// 5]. Returns an error if validation fails.
    ///
    /// [JSON Serialization]: https://datatracker.ietf.org/doc/html/rfc7515#section-7.2
    /// [RFC 5]: https://github.com/pgxn/rfcs/pull/5
    pub fn validate_payload(&mut self, meta: &Value) -> Result<(), Error> {
        self.validate_version_schema(meta, 2, "payload.schema.json")
    }

    fn validate_schema(&mut self, meta: &Value, schema: &str) -> Result<u8, Error> {
        let v = util::get_version(meta).ok_or(Error::UnknownSpec)?;
        self.validate_version_schema(meta, v, schema).map(|()| v)
    }

    fn validate_version_schema(&mut self, meta: &Value, v: u8, schema: &str) -> Result<(), Error> {
        let id = format!("{SCHEMA_BASE}{v}/{schema}");
        debug!(schema:display=id;"validate");

        let compiler = &mut self.compiler;
        let schemas = &mut self.schemas;
        let idx = compiler.compile(&id, schemas)?;
        schemas.validate(meta, idx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
