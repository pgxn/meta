use crate::error::Error;
/// Public but undocumented and un-exported module that creates a
/// boon::Compiler for use in validation and Tests.
use boon::Compiler;
use relative_path::{Component, RelativePath};
use serde_json::Value;

/// new returns a new boon::Compiler with the schema files loaded from `dir`
/// and configured to validate `path` and `license` formats.
pub fn new() -> Compiler {
    let schema_v1 = include_str!(concat!(env!("OUT_DIR"), "/pgxn-meta-v1.schemas.json"));
    let schema_v2 = include_str!(concat!(env!("OUT_DIR"), "/pgxn-meta-v2.schemas.json"));
    let mut compiler = spec_compiler();

    for str in [schema_v1, schema_v2] {
        for line in str.lines() {
            let schema: Value = serde_json::from_str(line).unwrap();
            let id = &schema["$id"]
                .as_str()
                .ok_or(Error::UnknownSchemaId)
                .unwrap();
            compiler.add_resource(id, schema.to_owned()).unwrap();
        }
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
fn is_path(v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let Value::String(s) = v else { return Ok(()) };

    let path = RelativePath::new(s);
    for c in path.components() {
        if c == Component::ParentDir {
            Err("references parent dir")?
        };
    }

    Ok(())
}

/// Returns an error if v is not a valid SPDX license expression.
fn is_license(v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let Value::String(s) = v else { return Ok(()) };
    _ = spdx::Expression::parse(s).map_err(crate::error::Error::License)?;
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
        for (name, invalid, err) in [
            ("parent", json!("../outside/path"), "references parent dir"),
            (
                "sub parent",
                json!("thing/../other"),
                "references parent dir",
            ),
        ] {
            match is_path(&invalid) {
                Ok(_) => panic!("{name} unexpectedly passed!"),
                Err(e) => assert_eq!(err, e.to_string(), "{name}"),
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
        for (name, invalid_license, reason) in [
            ("empty string", json!(""), spdx::error::Reason::Empty),
            ("zero", json!("0"), spdx::error::Reason::UnknownTerm),
            ("control chars", json!("\n\t"), spdx::error::Reason::Empty),
            (
                "parens",
                json!("()"),
                spdx::error::Reason::Unexpected(&["<license>", "("]),
            ),
            (
                "and",
                json!("AND"),
                spdx::error::Reason::Unexpected(&["<license>", "("]),
            ),
            (
                "or",
                json!("OR"),
                spdx::error::Reason::Unexpected(&["<license>", "("]),
            ),
        ] {
            match is_license(&invalid_license) {
                Ok(_) => panic!("{name} unexpectedly passed!"),
                Err(e) => assert_eq!(reason.to_string(), e.to_string(), "{name}"),
            }
        }
    }

    #[test]
    fn test_spec_compiler() -> Result<(), Error> {
        let mut c = spec_compiler();
        let id = "format";
        // Compile simple schema to validate license and path.
        c.add_resource(
            id,
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "format": "path",
                    },
                    "license": {
                        "type": "string",
                        "format": "license",
                    }
                }
            }),
        )?;

        let mut schemas = Schemas::new();
        let idx = c.compile(id, &mut schemas)?;

        for (name, json, err) in [
            (
                "empty license",
                json!({"license": ""}),
                "at '/license': '' is not valid license: empty expression",
            ),
            (
                "zero license",
                json!({"license": "0"}),
                "at '/license': '0' is not valid license: unknown term",
            ),
            (
                "parent path",
                json!({"path": "../foo"}),
                "'../foo' is not valid path: references parent dir",
            ),
        ] {
            match schemas.validate(&json, idx) {
                Ok(_) => panic!("{name} unexpectedly succeeded"),
                Err(e) => {
                    println!("{e}");
                    assert!(e.to_string().contains(err), "{name}")
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_new() -> Result<(), Error> {
        let mut compiler = new();

        for tc in [("v1", "widget.json"), ("v2", "typical-sql.json")] {
            let mut schemas = Schemas::new();
            let id = format!("https://pgxn.org/meta/{}/distribution.schema.json", tc.0);
            let idx = compiler.compile(&id, &mut schemas)?;

            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("corpus")
                .join(tc.0)
                .join(tc.1);
            let meta: Value = serde_json::from_reader(File::open(path)?)?;
            assert!(schemas.validate(&meta, idx).is_ok());
        }

        Ok(())
    }
}
