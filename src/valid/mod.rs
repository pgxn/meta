/*!
PGXN Distribution Metadata validation.

This module uses JSON Schema to validate PGXN `META.json` files.
It supports both the [v1] and [v2] specs.

# Example

``` rust
# use std::error::Error;
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
# Ok::<(), Box<dyn Error>>(())
```

  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3

*/
use std::{error::Error, fmt};

use crate::util;
use boon::{Compiler, Schemas};
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

/// Errors returned by Validator are ValidationError objects.
#[derive(Debug, PartialEq)]
pub enum ValidationError {
    /// UnknownSpec errors are returned when the validator cannot determine
    /// the version of the meta spec.
    UnknownSpec,
    /// UnknownID errors are returned by new() when a schema file has no `$id`
    /// property.
    UnknownID,
}

impl Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::UnknownSpec => write!(f, "Cannot determine meta-spec version"),
            ValidationError::UnknownID => write!(f, "No $id found in schema"),
        }
    }
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
    pub fn validate<'a>(&'a mut self, meta: &'a Value) -> Result<u8, Box<dyn Error + 'a>> {
        self.validate_schema(meta, "distribution.schema.json")
    }

    /// Validates PGXN release distribution metadata.
    ///
    /// On release, PGXN adds release metadata to the distribution `META.json`
    /// and publishes it separately so that clients can find and validate a release.
    /// The metadata includes the user who published the release, the release
    /// timestamp, and checksums for the distribution file. The v2 spec goes
    /// further by signing the release.
    ///
    /// This method validates the structure of such a release `META.json`
    /// file. Load one up into a serde_json::value::Value and pass it for
    /// validation. Returns the Meta spec version (1 or 2) on success and a
    /// validation error on failure.
    pub fn validate_release<'a>(&'a mut self, meta: &'a Value) -> Result<u8, Box<dyn Error + 'a>> {
        self.validate_schema(meta, "release.schema.json")
    }

    fn validate_schema<'a>(
        &'a mut self,
        meta: &'a Value,
        schema: &str,
    ) -> Result<u8, Box<dyn Error + 'a>> {
        let v = util::get_version(meta).ok_or(ValidationError::UnknownSpec)?;
        let id = format!("{SCHEMA_BASE}{v}/{schema}");

        let compiler = &mut self.compiler;
        let schemas = &mut self.schemas;
        let idx = compiler.compile(&id, schemas)?;
        schemas.validate(meta, idx)?;

        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::{error::Error, fs::File, path::PathBuf};
    use wax::Glob;

    #[test]
    fn test_corpus() -> Result<(), Box<dyn Error>> {
        let mut validator = Validator::default();

        for (version, release_patch) in [
            (
                1,
                json!({
                  "user": "theory",
                  "date": "2019-09-23T17:16:45Z",
                  "sha1": "0389be689af6992b4da520ec510d147bae411e8b",
                }),
            ),
            (
                2,
                json!({"release": {
                  "headers": ["eyJhbGciOiJFUzI1NiJ9"],
                  "signatures": [
                    "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
                  ],
                  "payload": {
                    "user": "theory",
                    "date": "2024-07-20T20:34:34Z",
                    "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
                    "digests": {
                      "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
                    }
                  }
                }}),
            ),
        ] {
            let v_dir = format!("v{version}");
            let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", &v_dir]
                .iter()
                .collect();
            let glob = Glob::new("*.json")?;

            for path in glob.walk(dir) {
                // Load metadata.
                let path = path?.into_path();
                let bn = path.file_name().unwrap().to_str().unwrap();
                let mut meta: Value = serde_json::from_reader(File::open(&path)?)?;

                // Should validate.
                match validator.validate(&meta) {
                    Err(e) => panic!("{v_dir}/{bn} validate failed: {e}"),
                    Ok(v) => assert_eq!(version, v, "{v_dir}/{bn} validate version"),
                };

                // Patch with release data and validate as release.
                json_patch::merge(&mut meta, &release_patch);
                match validator.validate_release(&meta) {
                    Err(e) => panic!("{v_dir}/{bn} validate_release failed: {e}"),
                    Ok(v) => assert_eq!(version, v, "{v_dir}/{bn} validate_release version"),
                };
            }
        }

        Ok(())
    }

    #[test]
    fn test_errors() {
        assert_eq!(
            format!("{}", ValidationError::UnknownSpec),
            "Cannot determine meta-spec version",
        );
        assert_eq!(
            format!("{}", ValidationError::UnknownID),
            "No $id found in schema",
        );
    }

    #[test]
    fn test_unknown_versions() -> Result<(), Box<dyn Error>> {
        let mut validator = Validator::new();

        for (name, json) in [
            ("no meta spec", json!({})),
            ("meta spec array", json!({"meta-spec": []})),
            ("no meta version", json!({"meta-spec": {}})),
            ("meta version bool", json!({"meta-spec": true})),
            ("bad meta version", json!({"meta-spec": {"version": "0.0"}})),
        ] {
            match validator.validate(&json) {
                Err(e) => assert_eq!(
                    "Cannot determine meta-spec version",
                    e.to_string(),
                    "{name} validate"
                ),
                Ok(_) => panic!("{name} validate unexpectedly succeeded"),
            }
            match validator.validate_release(&json) {
                Err(e) => assert_eq!(
                    "Cannot determine meta-spec version",
                    e.to_string(),
                    "{name} validate_release"
                ),
                Ok(_) => panic!("{name} validate validate_release succeeded"),
            }
        }

        Ok(())
    }

    fn load_minimal() -> Result<(Value, Value), Box<dyn Error>> {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
        let file = dir.join("v1").join("howto.json");
        let v1: Value = serde_json::from_reader(File::open(file)?)?;
        let file = dir.join("v2").join("minimal.json");
        let v2: Value = serde_json::from_reader(File::open(file)?)?;
        Ok((v1, v2))
    }

    #[test]
    fn test_invalid_distribution() -> Result<(), Box<dyn Error>> {
        let mut validator = Validator::new();
        let (v1, v2) = load_minimal()?;

        for (name, meta, patch, err) in [
            (
                "v1 no name",
                &v1,
                json!({"name": null}),
                "missing properties 'name'",
            ),
            (
                "v1 no version",
                &v1,
                json!({"version": null}),
                "missing properties 'version'",
            ),
            (
                "v1 invalid license",
                &v1,
                json!({"license": "lol no"}),
                "'/license': oneOf failed, none matched",
            ),
            (
                "v1 missing provides version",
                &v1,
                json!({"provides": {"pair": {"version": null}}}),
                "missing properties 'version'",
            ),
            (
                "v2 no name",
                &v2,
                json!({"name": null}),
                "missing properties 'name'",
            ),
            (
                "v2 no version",
                &v2,
                json!({"version": null}),
                "missing properties 'version'",
            ),
            (
                "v2 invalid license",
                &v2,
                json!({"license": "lol no"}),
                "'/license': 'lol no' is not valid license: lol no",
            ),
            (
                "v1 missing control",
                &v1,
                json!({"contents": {"extensions": {"pair": {"control": null}}}}),
                "'/contents': false schema",
            ),
        ] {
            let mut meta = meta.clone();
            json_patch::merge(&mut meta, &patch);
            match validator.validate(&meta) {
                Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
                Ok(_) => panic!("{name} validate unexpectedly succeeded"),
            };

            match validator.validate_release(&meta) {
                Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
                Ok(_) => panic!("{name} validate_release unexpectedly succeeded"),
            };
        }

        Ok(())
    }

    #[test]
    fn test_invalid_release() -> Result<(), Box<dyn Error>> {
        let mut validator = Validator::new();
        let (v1, v2) = load_minimal()?;
        for (name, meta, patch, err) in [
            (
                "v1 no sha",
                &v1,
                json!({"user": "xxx", "date": "2019-09-23T17:16:45Z"}),
                "missing properties 'sha1'",
            ),
            (
                "v1 no user",
                &v1,
                json!({"sha1": "0389be689af6992b4da520ec510d147bae411e8b", "date": "2019-09-23T17:16:45Z"}),
                "missing properties 'user'",
            ),
            (
                "v1 no date",
                &v1,
                json!({"user": "xxx", "sha1": "0389be689af6992b4da520ec510d147bae411e8b"}),
                "missing properties 'date'",
            ),
            (
                "v2 no release",
                &v2,
                json!({}),
                "missing properties 'release'",
            ),
            (
                "v2 no release user",
                &v2,
                json!({"release": {
                  "headers": ["eyJhbGciOiJFUzI1NiJ9"],
                  "signatures": [
                    "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
                  ],
                  "payload": {
                    "date": "2024-07-20T20:34:34Z",
                    "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
                    "digests": {
                      "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
                    }
                  }
                }}),
                "'/release/payload': missing properties 'user'",
            ),
            (
                "v2 no headers",
                &v2,
                json!({"release": {
                  "headers": [],
                  "signatures": [
                    "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
                  ],
                  "payload": {
                    "user": "xxx",
                    "date": "2024-07-20T20:34:34Z",
                    "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
                    "digests": {
                      "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
                    }
                  }
                }}),
                "'/release/headers': minimum 1 items required, but got 0 items",
            ),
        ] {
            let mut meta = meta.clone();
            json_patch::merge(&mut meta, &patch);

            match validator.validate_release(&meta) {
                Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
                Ok(_) => panic!("{name} validate_release unexpectedly succeeded"),
            };
        }

        Ok(())
    }
}
