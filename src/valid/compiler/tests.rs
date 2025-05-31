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
        json!("./relative/path"),
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
        (
            "parent",
            json!("../outside/path"),
            "references parent directory",
        ),
        (
            "current",
            json!("/./outside/path"),
            "references current directory",
        ),
        (
            "sub parent",
            json!("thing/../other"),
            "references parent directory",
        ),
    ] {
        match is_path(&invalid) {
            Ok(_) => panic!("{name} unexpectedly passed!"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }
}

#[test]
fn test_glob() {
    // Test valid globs.
    for valid in [
        json!(".gitignore"),
        json!(".git*"),
        json!("/git*"),
        json!("./git*"),
        json!(""),
        json!("README.*"),
        json!("**/*.(?i){jpg,jpeg}"),
        json!("**/{*.{go,rs}}"),
        json!("src/**/*.rs"),
    ] {
        if let Err(e) = is_glob(&valid) {
            panic!("{} failed: {e}", valid);
        }
    }

    // Test invalid paths.
    for (name, invalid) in [
        ("parent", json!("../*.c")),
        ("current", json!("/./*.c")),
        ("sub parent", json!("/**/../passwd")),
    ] {
        match is_glob(&invalid) {
            Ok(_) => panic!("{name} unexpectedly passed!"),
            Err(e) => assert_eq!(
                "references parent or current directory",
                e.to_string(),
                "{name}"
            ),
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
    // Compile simple schema to validate license, path, and glob.
    c.add_resource(
        id,
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "format": "path",
                },
                "glob": {
                    "type": "string",
                    "format": "glob",
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
            "'../foo' is not valid path: references parent directory",
        ),
        (
            "parent glob",
            json!({"glob": "../*.c"}),
            "'../*.c' is not valid glob: references parent or current directory",
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
