//! The valid module provides pgxn_meta validation.
use std::path::Path;
use std::{error::Error, fmt};

use boon::{Compiler, Schemas};
use serde_json::Value;

/// PGXN Meta validator.
pub struct Validator {
    compiler: Compiler,
    schemas: Schemas,
}

/// Errors returned by Validator are ValidationError objects.
#[derive(Debug)]
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
const SCHEMA_BASE: &str = "https://pgxn.org/meta/v";

impl Validator {
    /// Validator constructor.
    ///
    /// new creates and returns a new Validator with the schemas loaded from
    /// `dir`.
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Validator, Box<dyn Error>> {
        let compiler = super::compiler::new(dir)?;
        let schemas = Schemas::new();
        Ok(Validator { compiler, schemas })
    }

    /// Validates a PGXN Meta document.
    ///
    /// Load a `META.json` file into a serde_json::value::Value and pass it
    /// for validation. Returns true on success and a validation error on
    /// failure.
    pub fn validate<'a>(&'a mut self, meta: &'a Value) -> Result<bool, Box<dyn Error + '_>> {
        let map = meta.as_object().ok_or(ValidationError::UnknownSpec)?;
        let version = map
            .get("meta-spec")
            .ok_or(ValidationError::UnknownSpec)?
            .as_object()
            .ok_or(ValidationError::UnknownSpec)?
            .get("version")
            .ok_or(ValidationError::UnknownSpec)?
            .as_str()
            .ok_or(ValidationError::UnknownSpec)?;

        let v = match &version[0..2] {
            "1." => 1,
            "2." => 2,
            _ => return Err(Box::new(ValidationError::UnknownSpec)),
        };
        let id = format!("{SCHEMA_BASE}{v}/distribution.schema.json");

        let compiler = &mut self.compiler;
        let schemas = &mut self.schemas;
        let idx = compiler.compile(&id, schemas)?;
        schemas.validate(meta, idx)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::error::Error;
    use std::fs::File;
    use std::path::PathBuf;
    use wax::Glob;

    #[test]
    fn test_corpus() -> Result<(), Box<dyn Error>> {
        let schemas_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "schema"].iter().collect();
        let mut validator = Validator::new(schemas_dir)?;

        for v_dir in ["v1", "v2"] {
            let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "corpus", v_dir]
                .iter()
                .collect();
            let glob = Glob::new("*.json")?;

            for path in glob.walk(dir) {
                let path = path?.into_path();
                let meta: Value = serde_json::from_reader(File::open(&path)?)?;
                if let Err(e) = validator.validate(&meta) {
                    panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap());
                }
                println!("Example {v_dir}/{:?} ok", path.file_name().unwrap());
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
    fn test_invalid_schemas() -> Result<(), Box<dyn Error>> {
        let schemas_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "schema"].iter().collect();
        let mut validator = Validator::new(schemas_dir)?;

        for tc in [
            ("no meta spec", json!({})),
            ("meta spec array", json!({"meta-spec": []})),
            ("no meta version", json!({"meta-spec": {}})),
            ("meta version bool", json!({"meta-spec": true})),
            ("bad meta version", json!({"meta-spec": {"version": "0.0"}})),
        ] {
            let res = validator.validate(&tc.1);
            assert!(res.is_err());
        }

        Ok(())
    }
}
