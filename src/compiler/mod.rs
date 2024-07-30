use relative_path::{Component, RelativePath};
/// Public but undocumented and un-exported module that creates a
/// boon::Compiler for use in validation and Tests.
use std::error::Error;
use std::fs::File;
use std::path::Path;

use boon::Compiler;
use serde_json::Value;
use wax::Glob;

/// new returns a new boon::compiler with the schema files loaded from `dir`
/// and configured to validate `path` and `license` formats.
pub fn new<P: AsRef<Path>>(dir: P) -> Result<Compiler, Box<dyn Error>> {
    let mut compiler = spec_compiler();

    let glob = Glob::new("**/*.schema.json")?;
    for path in glob.walk(dir) {
        let schema: Value = serde_json::from_reader(File::open(path?.into_path())?)?;
        let s = &schema["$id"]
            .as_str()
            .ok_or(super::valid::ValidationError::UnknownID)?;
        compiler.add_resource(s, schema.to_owned())?;
    }

    Ok(compiler)
}

/// Creates a new boon::compiler with format assertions enabled and validation
/// for the custom `path` and `license` formats.
fn spec_compiler() -> Compiler {
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
    use serde_json::json;

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
}
