use relative_path::{Component, RelativePath};
/// Public but undocumented and un-exported module that creates a
/// boon::Compiler for use in validation and Tests.
use std::error::Error;

use boon::Compiler;
use serde_json::Value;

/// new returns a new boon::Compiler with the schema files loaded from `dir`
/// and configured to validate `path` and `license` formats.
pub fn new() -> Compiler {
    let schema_v1 = include_str!(concat!(env!("OUT_DIR"), "/pgxn-meta-v1.schema.json"));
    let schema_v2 = include_str!(concat!(env!("OUT_DIR"), "/pgxn-meta-v2.schema.json"));
    let mut compiler = spec_compiler();

    for str in [schema_v1, schema_v2] {
        let schema: Value = serde_json::from_str(str).unwrap();
        let id = &schema["$id"]
            .as_str()
            .ok_or(super::valid::ValidationError::UnknownID)
            .unwrap();
        compiler.add_resource(id, schema.to_owned()).unwrap();
    }

    compiler
}

/// Creates a new boon::compiler with format assertions enabled and validation
/// for the custom `path` and `license` formats.
pub fn spec_compiler() -> Compiler {
    let mut compiler = Compiler::new();
    compiler.enable_format_assertions();
    compiler.register_format(boon::Format {
        name: "path",
        func: is_path,
    });
    compiler.register_format(boon::Format {
        name: "license",
        func: is_license,
    });
    compiler
}

/// Returns an error if v is not a valid path.
fn is_path(v: &Value) -> Result<(), Box<dyn Error>> {
    let Value::String(s) = v else {
        Err("not a string")?
    };

    let path = RelativePath::new(s);
    for c in path.components() {
        if c == Component::ParentDir {
            Err("parent dir")?
        };
    }

    Ok(())
}

/// Returns an error if vi is not a valid SPDX license expression.
fn is_license(v: &Value) -> Result<(), Box<dyn Error>> {
    let Value::String(s) = v else {
        Err("not a string")?
    };
    _ = spdx::Expression::parse(s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boon::Schemas;
    use serde_json::json;
    use std::{fs::File, path::Path};

    #[test]
    fn test_path() {
        // Test valid paths.
        for valid in [
            json!("\\foo.md"),
            json!("this\\and\\that.txt"),
            json!("/absolute/path"),
            json!(""),
            json!("C:\\foo"),
            json!("README.txt"),
            json!(".git"),
            json!("src/pair.c"),
            json!(".github/workflows/"),
            json!("this\\\\and\\\\that.txt"),
        ] {
            if let Err(e) = is_path(&valid) {
                panic!("{} failed: {e}", valid);
            }
        }

        // Test invalid paths.
        for invalid in [
            json!("../outside/path"),
            json!("thing/../other"),
            json!({}),
            json!([]),
            json!(true),
            json!(null),
            json!(42),
        ] {
            if is_path(&invalid).is_ok() {
                panic!("{} unexpectedly passed!", invalid)
            }
        }
    }

    #[test]
    fn test_license() {
        // Test valid relative licenses.
        for valid_license in [
            json!("MIT"),
            json!("PostgreSQL"),
            json!("Apache-2.0 OR MIT"),
            json!("Apache-2.0 OR MIT OR PostgreSQL"),
            json!("Apache-2.0 AND MIT"),
            json!("MIT OR Apache-2.0 AND BSD-2-Clause"),
            json!("(MIT AND (LGPL-2.1-or-later OR BSD-3-Clause))"),
            json!("((Apache-2.0 WITH LLVM-exception) OR Apache-2.0) AND OpenSSL OR MIT"),
            json!("Apache-2.0 WITH LLVM-exception OR Apache-2.0 AND (OpenSSL OR MIT)"),
            json!("Apache-2.0 WITH LLVM-exception OR (Apache-2.0 AND OpenSSL) OR MIT"),
            json!("((((Apache-2.0 WITH LLVM-exception) OR (Apache-2.0)) AND (OpenSSL)) OR (MIT))"),
            json!("CDDL-1.0+"),
            json!("LicenseRef-23"),
            json!("LicenseRef-MIT-Style-1"),
            json!("DocumentRef-spdx-tool-1.2:LicenseRef-MIT-Style-2"),
        ] {
            if let Err(e) = is_license(&valid_license) {
                panic!("{} failed: {e}", valid_license);
            }
        }

        // Test invalid licenses.
        for invalid_license in [
            json!(""),
            json!(null),
            json!("0"),
            json!(0),
            json!("\n\t"),
            json!("()"),
            json!("AND"),
            json!("OR"),
        ] {
            if is_license(&invalid_license).is_ok() {
                panic!("{} unexpectedly passed!", invalid_license)
            }
        }
    }

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let mut compiler = new();

        for tc in [("v1", "widget.json"), ("v2", "typical-sql.json")] {
            let mut schemas = Schemas::new();
            let id = format!("https://pgxn.org/meta/{}/distribution.schema.json", tc.0);
            let idx = compiler.compile(&id, &mut schemas)?;

            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("corpus")
                .join(tc.0)
                .join(tc.1);
            let meta: Value = serde_json::from_reader(File::open(path)?)?;
            assert!(schemas.validate(&meta, idx).is_ok());
        }

        Ok(())
    }
}
