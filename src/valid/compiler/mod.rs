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
/// for the custom `path`, `glob`, and `license` formats.
pub fn spec_compiler() -> Compiler {
    let mut compiler = Compiler::new();
    compiler.enable_format_assertions();
    compiler.register_format(boon::Format {
        name: "path",
        func: is_path,
    });
    compiler.register_format(boon::Format {
        name: "glob",
        func: is_glob,
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

    let path = RelativePath::new(s.strip_prefix("./").unwrap_or(s));
    for c in path.components() {
        match c {
            Component::ParentDir => Err("references parent directory")?,
            Component::CurDir => Err("references current directory")?,
            _ => (),
        }
    }

    Ok(())
}

/// Returns an error if v is not a valid glob.
fn is_glob(v: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let Value::String(s) = v else { return Ok(()) };

    // XXX Use https://docs.rs/glob/latest/glob/struct.Pattern.html instead?
    let path = wax::Glob::new(s.strip_prefix("./").unwrap_or(s))?;
    if path.has_semantic_literals() {
        Err("references parent or current directory")?
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
mod tests;
