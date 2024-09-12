use std::error::Error;

use boon::Schemas;
use serde_json::{json, Map, Value};

// importing common module.
use super::common::*;

const SCHEMA_VERSION: u8 = 1;

#[test]
fn test_schema_v1() -> Result<(), Box<dyn Error>> {
    test_schema_version(SCHEMA_VERSION)
}

#[test]
fn test_v1_term() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the term schema.
    let compiler = new_compiler("schema/v1")?;
    test_term_schema(compiler, SCHEMA_VERSION)
}

#[test]
fn test_v1_tags() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the tags schema.
    let compiler = new_compiler("schema/v1")?;
    test_tags_schema(compiler, SCHEMA_VERSION)
}

#[test]
fn test_v1_version() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the version schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "version");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_version in VALID_SEMVERS {
        let vv = json!(valid_version);
        if let Err(e) = schemas.validate(&vv, idx) {
            panic!("extension {} failed: {e}", valid_version);
        }
    }

    for invalid_version in INVALID_SEMVERS {
        let iv = json!(invalid_version);
        if schemas.validate(&iv, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_version)
        }
    }

    Ok(())
}

#[test]
fn test_v1_version_range() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the version_range schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "version_range");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_version in VALID_SEMVERS {
        for op in ["", "==", "!=", ">", "<", ">=", "<="] {
            for append in [
                "",
                ",<= 1.1.2+meta",
                ",>= 2.0.0, 1.5.6",
                ", >1.2.0, != 12.0.0, < 19.2.0",
            ] {
                let range = json!(format!("{}{}{}", op, valid_version, append));
                if let Err(e) = schemas.validate(&range, idx) {
                    panic!("extension {} failed: {e}", range);
                }

                // Version zero must not appear in a range.
                let range = json!(format!("{}{}{},0", op, valid_version, append));
                if schemas.validate(&range, idx).is_ok() {
                    panic!("{} unexpectedly passed!", range)
                }
            }
        }

        // Test with unknown operators.
        for bad_op in ["!", "=", "<>", "=>", "=<"] {
            let range = json!(format!("{}{}", bad_op, valid_version));
            if schemas.validate(&range, idx).is_ok() {
                panic!("{} unexpectedly passed!", range)
            }
        }
    }

    // Bare integer 0 allowed.
    let zero = json!(0);
    if let Err(e) = schemas.validate(&zero, idx) {
        panic!("extension {} failed: {e}", zero);
    }

    // But version 0 cannot appear with any range operator or in any range.
    for op in ["", "==", "!=", ">", "<", ">=", "<="] {
        let range = json!(format!("{op}0"));
        if let Err(e) = schemas.validate(&range, idx) {
            panic!("extension {} failed: {e}", range);
        }
    }

    for invalid_version in INVALID_SEMVERS {
        for op in ["", "==", "!=", ">", "<", ">=", "<="] {
            for append in [
                "",
                ",<= 1.1.2+meta",
                ",>= 2.0.0, 1.5.6",
                ", >1.2.0, != 12.0.0, < 19.2.0",
            ] {
                let range = json!(format!("{}{}{}", op, invalid_version, append));
                if schemas.validate(&range, idx).is_ok() {
                    panic!("{} unexpectedly passed!", invalid_version)
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_v1_license() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the license schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "license");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid license values.
    for valid_license in [
        json!("agpl_3"),
        json!("apache_1_1"),
        json!("apache_2_0"),
        json!("artistic_1"),
        json!("artistic_2"),
        json!("bsd"),
        json!("freebsd"),
        json!("gfdl_1_2"),
        json!("gfdl_1_3"),
        json!("gpl_1"),
        json!("gpl_2"),
        json!("gpl_3"),
        json!("lgpl_2_1"),
        json!("lgpl_3_0"),
        json!("mit"),
        json!("mozilla_1_0"),
        json!("mozilla_1_1"),
        json!("openssl"),
        json!("perl_5"),
        json!("postgresql"),
        json!("qpl_1_0"),
        json!("ssleay"),
        json!("sun"),
        json!("zlib"),
        json!("open_source"),
        json!("restricted"),
        json!("unrestricted"),
        json!("unknown"),
        json!(["postgresql", "perl_5"]),
        json!({"foo": "https://foo.com"}),
        json!({"foo": "https://foo.com", "bar": "https://bar.com"}),
    ] {
        if let Err(e) = schemas.validate(&valid_license, idx) {
            panic!("license {} failed: {e}", valid_license);
        }
    }

    // Test invalid license values.
    for invalid_license in [
        json!("nonesuch"),
        json!("crank"),
        json!(""),
        json!(true),
        json!(false),
        json!(null),
        json!(["nonesuch"]),
        json!([]),
        json!({}),
        json!({"foo": ":hello"}),
        json!(["mit", "mit"]),
    ] {
        if schemas.validate(&invalid_license, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_license)
        }
    }

    Ok(())
}

#[test]
fn test_v1_provides() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the provides schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "provides");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_provides in [
        (
            "required fields",
            json!({"pgtap": {
                "file": "widget.sql",
                "version": "0.26.0",
            }}),
        ),
        (
            "all fields",
            json!({"pgtap": {
                "docfile": "foo/bar.txt",
                "abstract": "This and that",
                "file": "widget.sql",
                "version": "0.26.0",
            }}),
        ),
        (
            "x field",
            json!({"pgtap": {
                "file": "widget.sql",
                "version": "0.26.0",
                "x_foo": 1,
            }}),
        ),
        (
            "X field",
            json!({"pgtap": {
                "file": "widget.sql",
                "version": "0.26.0",
                "X_foo": 1,
            }}),
        ),
        (
            "two extensions",
            json!({
                "pgtap": {
                    "file": "widget.sql",
                    "version": "0.26.0",
                },
                "pgtap_common": {
                    "file": "common.sql",
                    "version": "0.26.0",
                },
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_provides.1, idx) {
            panic!("{} failed: {e}", valid_provides.0);
        }
    }

    for invalid_provides in [
        // Basics
        ("array", json!([])),
        ("string", json!("crank")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        (
            "invalid key",
            json!({"x": {"file": "x.sql", "version": "1.0.0"}}),
        ),
        (
            "invalid field",
            json!({"xy": {"file": "x.sql", "version": "1.0.0", "foo": "foo"}}),
        ),
        ("no file", json!({"pgtap": {"version": "0.26.0"}})),
        (
            "invalid version",
            json!({"x": {"file": "x.sql", "version": "1.0"}}),
        ),
        (
            "null file",
            json!({"x": {"file": null, "version": "1.0.0"}}),
        ),
        (
            "bare x_",
            json!({"x": {"file": "x.txt", "version": "1.0.0", "x_": 0}}),
        ),
    ] {
        if schemas.validate(&invalid_provides.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_provides.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_extension() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the extension schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "extension");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_extension in [
        (
            "required fields",
            json!({
                "file": "widget.sql",
                "version": "0.26.0",
            }),
        ),
        (
            "with abstract",
            json!({
                "file": "widget.sql",
                "version": "0.26.0",
                "abstract": "This and that",
            }),
        ),
        (
            "all fields",
            json!({
                "docfile": "foo/bar.txt",
                "abstract": "This and that",
                "file": "widget.sql",
                "version": "0.26.0",
            }),
        ),
        (
            "x field",
            json!({
                "version": "0.26.0",
                "file": "widget.sql",
                "x_hi": true,
            }),
        ),
        (
            "X field",
            json!({
                "version": "0.26.0",
                "file": "widget.sql",
                "X_bar": 42,
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_extension.1, idx) {
            panic!("extension {} failed: {e}", valid_extension.0);
        }
    }

    for invalid_extension in [
        // Basics
        ("array", json!([])),
        ("string", json!("crank")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        (
            "invalid field",
            json!({"file": "widget.sql", "version": "0.26.0", "foo": "hi", }),
        ),
        (
            "bare x_",
            json!({
                "file": "widget.sql",
                "version": "0.26.0",
                "x_": "hi",
            }),
        ),
        // File
        ("no file", json!({"version": "0.26.0"})),
        ("null file", json!({"file": null, "version": "0.26.0"})),
        (
            "empty string file",
            json!({"file": "", "version": "0.26.0"}),
        ),
        ("number file", json!({"file": 42, "version": "0.26.0"})),
        ("bool file", json!({"file": true, "version": "0.26.0"})),
        ("array file", json!({"file": ["hi"], "version": "0.26.0"})),
        ("object file", json!({"file": {}, "version": "0.26.0"})),
        // Version
        ("no version", json!({"file": "widget.sql"})),
        (
            "invalid version",
            json!({"file": "widget.sql", "version": "1.0"}),
        ),
        (
            "null version",
            json!({"file": "widget.sql", "version": null}),
        ),
        (
            "empty version",
            json!({"file": "widget.sql", "version": ""}),
        ),
        (
            "number version",
            json!({"file": "widget.sql", "version": 42}),
        ),
        (
            "bool version",
            json!({"file": "widget.sql", "version": false}),
        ),
        (
            "array version",
            json!({"file": "widget.sql", "version": ["1.0.0"]}),
        ),
        (
            "objet version",
            json!({"file": "widget.sql", "version": {}}),
        ),
        // Abstract
        (
            "empty abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": ""}),
        ),
        (
            "null abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": null}),
        ),
        (
            "number abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": 42}),
        ),
        (
            "bool abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": true}),
        ),
        (
            "array abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": ["hi"]}),
        ),
        (
            "object abstract",
            json!({"file": "widget.sql", "version": "1.0.0", "abstract": {}}),
        ),
        // Docfile
        (
            "empty docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": ""}),
        ),
        (
            "null docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": null}),
        ),
        (
            "number docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": 42}),
        ),
        (
            "bool docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": true}),
        ),
        (
            "array docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": ["hi"]}),
        ),
        (
            "object docfile",
            json!({"file": "widget.sql", "version": "1.0.0", "docfile": {}}),
        ),
    ] {
        if schemas.validate(&invalid_extension.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_extension.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_maintainer() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "maintainer");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_maintainer in [
        ("min length", json!("x")),
        ("min length_array", json!(["x"])),
        (
            "name and email",
            json!("David E. Wheeler <theory@pgxn.org>"),
        ),
        (
            "two names and emails",
            json!([
                "David E. Wheeler <theory@pgxn.org>",
                "Josh Berkus <jberkus@pgxn.org>"
            ]),
        ),
        ("space", json!("hi there")),
        ("slash", json!("hi/there")),
        ("backslash", json!("hi\\there")),
        ("null byte", json!("hi\x00there")),
    ] {
        if let Err(e) = schemas.validate(&valid_maintainer.1, idx) {
            panic!("extension {} failed: {e}", valid_maintainer.0);
        }
    }

    for invalid_maintainer in [
        ("empty array", json!([])),
        ("empty string", json!("")),
        ("empty string in array", json!(["hi", ""])),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("dupe", json!(["x", "x"])),
    ] {
        if schemas.validate(&invalid_maintainer.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_maintainer.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_meta_spec() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "meta-spec");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_meta_spec in [
        ("version 1.0.0 only", json!({"version": "1.0.0"})),
        ("version 1.0.1 only", json!({"version": "1.0.1"})),
        ("version 1.0.2 only", json!({"version": "1.0.2"})),
        ("version 1.0.99 only", json!({"version": "1.0.99"})),
        ("x key", json!({"version": "1.0.99", "x_y": true})),
        ("X key", json!({"version": "1.0.99", "X_x": true})),
        (
            "version plus https URL",
            json!({"version": "1.0.0", "url": "https://pgxn.org/meta/spec.txt"}),
        ),
        (
            "version plus http URL",
            json!({"version": "1.0.0", "url": "http://pgxn.org/meta/spec.txt"}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_meta_spec.1, idx) {
            panic!("extension {} failed: {e}", valid_meta_spec.0);
        }
    }

    for invalid_meta_spec in [
        ("array", json!([])),
        ("string", json!("1.0.0")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("unknown field", json!({"version": "1.0.0", "foo": "hi"})),
        ("bare x_", json!({"version": "1.0.0", "x_": "hi"})),
        ("version 1.1.0", json!({"version": "1.1.0"})),
        ("version 2.0.0", json!({"version": "2.0.0"})),
        (
            "no_version",
            json!({"url": "https://pgxn.org/meta/spec.txt"}),
        ),
        (
            "invalid url",
            json!({"version": "1.0.1", "url": "https://pgxn.org/meta/spec.html"}),
        ),
    ] {
        if schemas.validate(&invalid_meta_spec.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_meta_spec.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_bugtracker() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "bugtracker");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_bugtracker in [
        ("web only", json!({"web": "https://foo.com"})),
        ("mailto only", json!({"mailto": "hi@example.com"})),
        (
            "web and mailto",
            json!({"web": "https://foo.com", "mailto": "hi@example.com"}),
        ),
        ("x key", json!({"web": "https://foo.com", "x_q": true})),
        ("X key", json!({"web": "https://foo.com", "X_hi": true})),
    ] {
        if let Err(e) = schemas.validate(&valid_bugtracker.1, idx) {
            panic!("extension {} failed: {e}", valid_bugtracker.0);
        }
    }

    for invalid_bugtracker in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("unknown field", json!({"web": "https://foo.com", "foo": 0})),
        ("bare x_", json!({"web": "https://foo.com", "x_": 0})),
        ("web array", json!({"web": []})),
        ("web object", json!({"web": {}})),
        ("web bool", json!({"web": true})),
        ("web null", json!({"web": null})),
        ("web number", json!({"web": 52})),
        ("mailto array", json!({"mailto": []})),
        ("mailto object", json!({"mailto": {}})),
        ("mailto bool", json!({"mailto": true})),
        ("mailto null", json!({"mailto": null})),
        ("mailto number", json!({"mailto": 52})),
        ("invalid web url", json!({"web": "3ttp://a.com"})),
        ("missing required", json!({"x_y": "https://foo.com"})),
    ] {
        if schemas.validate(&invalid_bugtracker.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_bugtracker.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_no_index() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "no_index");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_no_index in [
        ("file only", json!({"file": ["x.txt"]})),
        ("file string", json!({"file": "x.txt"})),
        ("directory only", json!({"directory": [".git"]})),
        ("directory string", json!({"directory": ".git"})),
        (
            "both arrays",
            json!({"file": ["x.txt"], "directory": [".git"]}),
        ),
        (
            "file string dir array",
            json!({"file": "x.txt", "directory": [".git"]}),
        ),
        (
            "file array dir string",
            json!({"file": ["x.txt"], "directory": ".git"}),
        ),
        ("two files", json!({"file": ["x.txt", "y.md"]})),
        ("two dirs", json!({"directory": ["x", "y"]})),
        ("x_ field", json!({"file": ["x.txt"], "x_Y": 0})),
        ("X_ field", json!({"file": ["x.txt"], "X_y": 0})),
    ] {
        if let Err(e) = schemas.validate(&valid_no_index.1, idx) {
            panic!("extension {} failed: {e}", valid_no_index.0);
        }
    }

    for invalid_no_index in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("empty file", json!({"file": []})),
        ("empty file string", json!({"file": [""]})),
        ("file object", json!({"file": {}})),
        ("file bool", json!({"file": true})),
        ("file null", json!({"file": null})),
        ("empty directory", json!({"directory": []})),
        ("empty directory string", json!({"directory": [""]})),
        ("directory object", json!({"directory": {}})),
        ("directory bool", json!({"directory": true})),
        ("directory null", json!({"directory": null})),
        ("unknown field", json!({"file": ["x"], "hi": 0})),
        ("bare x_", json!({"file": ["x"], "x_": 0})),
        ("dupe", json!({"file": ["x", "x"]})),
        ("dupe", json!({"dir": ["x", "x"]})),
        ("missing required", json!({"x_y": 0})),
    ] {
        if schemas.validate(&invalid_no_index.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_no_index.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_prereq_relationship() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "prereq_relationship");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_prereq_relationship in [
        ("single prereq", json!({"citext": "2.0.0"})),
        ("two prereqs", json!({"citext": "2.0.0", "pgtap": "0.98.3"})),
        ("version op", json!({"citext": ">=2.0.0"})),
        ("version zero", json!({"citext": 0})),
        ("version zero string", json!({"citext": "0"})),
        (
            "version range",
            json!({"citext": ">= 1.2.0, != 1.5.0, < 2.0.0"}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_prereq_relationship.1, idx) {
            panic!("extension {} failed: {e}", valid_prereq_relationship.0);
        }
    }

    for invalid_prereq_relationship in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("short term", json!({"x": "1.0.0"})),
        ("null byte term", json!({"x\x00y": "1.0.0"})),
        ("invalid version", json!({"xy": "1.0"})),
        ("invalid version range", json!({"xy": "!1.0.0"})),
        ("number value", json!({"xx": 42})),
        ("empty string value", json!({"xx": ""})),
        ("null value", json!({"xx": null})),
        ("bool value", json!({"xx": true})),
        ("array value", json!({"xx": []})),
    ] {
        if schemas
            .validate(&invalid_prereq_relationship.1, idx)
            .is_ok()
        {
            panic!("{} unexpectedly passed!", invalid_prereq_relationship.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_prereq_phase() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "prereq_phase");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_prereq_phase in [
        ("requires", json!({"requires": {"citext": "2.0.0"}})),
        ("recommends", json!({"recommends": {"citext": "2.0.0"}})),
        ("suggests", json!({"suggests": {"citext": "2.0.0"}})),
        ("conflicts", json!({"conflicts": {"citext": "2.0.0"}})),
        (
            "two phases",
            json!({
                "requires": {"citext": "1.0.0"},
                "recommends": {"citext": "2.0.0"},
            }),
        ),
        (
            "three phases",
            json!({
                "requires": {"citext": "1.0.0"},
                "recommends": {"citext": "2.0.0"},
                "suggests": {"citext": "3.0.0"},
            }),
        ),
        (
            "four phases",
            json!({
                "requires": {"citext": "1.0.0"},
                "recommends": {"citext": "2.0.0"},
                "suggests": {"citext": "3.0.0"},
                "conflicts": { "alligator": 0}
            }),
        ),
        ("bare zero", json!({"requires": {"citext": 0}})),
        ("string zero", json!({"requires": {"citext": "0"}})),
        ("range op", json!({"requires": {"citext": "==2.0.0"}})),
        (
            "range",
            json!({"requires": {"citext": ">= 1.2.0, != 1.5.0, < 2.0.0"}}),
        ),
        (
            "x_ field",
            json!({"requires": {"citext": "2.0.0"}, "x_y": 1}),
        ),
        (
            "X_ field",
            json!({"requires": {"citext": "2.0.0"}, "X_y": 1}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_prereq_phase.1, idx) {
            panic!("extension {} failed: {e}", valid_prereq_phase.0);
        }
    }

    for invalid_prereq_phase in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("only x_ property", json!({"x_y": 0})),
        (
            "unknown property",
            json!({"requires": {"citext": "2.0.0"}, "foo": 0}),
        ),
        (
            "bare x_ property",
            json!({"requires": {"citext": "2.0.0"}, "x_": 0}),
        ),
        // requires
        ("requires array", json!({"requires": ["2.0.0"]})),
        ("requires object", json!({"requires": {}})),
        ("requires string", json!({"requires": "2.0.0"})),
        ("requires bool", json!({"requires": true})),
        ("requires number", json!({"requires": 42})),
        ("requires null", json!({"requires": null})),
        // recommends
        ("recommends array", json!({"recommends": ["2.0.0"]})),
        ("recommends object", json!({"recommends": {}})),
        ("recommends string", json!({"recommends": "2.0.0"})),
        ("recommends bool", json!({"recommends": true})),
        ("recommends number", json!({"recommends": 42})),
        ("recommends null", json!({"recommends": null})),
        // suggests
        ("suggests array", json!({"suggests": ["2.0.0"]})),
        ("suggests object", json!({"suggests": {}})),
        ("suggests string", json!({"suggests": "2.0.0"})),
        ("suggests bool", json!({"suggests": true})),
        ("suggests number", json!({"suggests": 42})),
        ("suggests null", json!({"suggests": null})),
        // conflicts
        ("conflicts array", json!({"conflicts": ["2.0.0"]})),
        ("conflicts object", json!({"conflicts": {}})),
        ("conflicts string", json!({"conflicts": "2.0.0"})),
        ("conflicts bool", json!({"conflicts": true})),
        ("conflicts number", json!({"conflicts": 42})),
        ("conflicts null", json!({"conflicts": null})),
    ] {
        if schemas.validate(&invalid_prereq_phase.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_prereq_phase.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_prereqs() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "prereqs");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_prereqs in [
        (
            "runtime",
            json!({"runtime": {"requires": {"citext": "2.0.0"}}}),
        ),
        ("build", json!({"build": {"requires": {"citext": "2.0.0"}}})),
        ("test", json!({"test": {"requires": {"citext": "2.0.0"}}})),
        (
            "configure",
            json!({"configure": {"requires": {"citext": "2.0.0"}}}),
        ),
        (
            "develop",
            json!({"develop": {"requires": {"citext": "2.0.0"}}}),
        ),
        (
            "two phases",
            json!({
                "build": {"requires": {"citext": "2.0.0"}},
                "test": {"requires": {"citext": "2.0.0"}}
            }),
        ),
        (
            "three phases",
            json!({
                "configure": {"requires": {"citext": "2.0.0"}},
                "build": {"requires": {"citext": "2.0.0"}},
                "test": {"requires": {"citext": "2.0.0"}}
            }),
        ),
        (
            "four phases",
            json!({
                "configure": {"requires": {"citext": "2.0.0"}},
                "build": {"requires": {"citext": "2.0.0"}},
                "test": {"requires": {"citext": "2.0.0"}},
                "runtime": {"requires": {"citext": "2.0.0"}},
            }),
        ),
        (
            "all phases",
            json!({
                "configure": {"requires": {"citext": "2.0.0"}},
                "build": {"requires": {"citext": "2.0.0"}},
                "test": {"requires": {"citext": "2.0.0"}},
                "runtime": {"requires": {"citext": "2.0.0"}},
                "develop": {"requires": {"citext": "2.0.0"}},
            }),
        ),
        (
            "runtime plus custom field",
            json!({
                "runtime": {"requires": {"citext": "2.0.0"}},
                "x_Y": 0,
            }),
        ),
        (
            "all phases plus custom",
            json!({
                "configure": {"requires": {"citext": "2.0.0"}},
                "build": {"requires": {"citext": "2.0.0"}},
                "test": {"requires": {"citext": "2.0.0"}},
                "runtime": {"requires": {"citext": "2.0.0"}},
                "develop": {"requires": {"citext": "2.0.0"}},
                "x_Y": 0,
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_prereqs.1, idx) {
            panic!("extension {} failed: {e}", valid_prereqs.0);
        }
    }

    for invalid_prereqs in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("only x_ field", json!({"x_y": 0})),
        (
            "bare x_ property",
            json!({
                "test": {"requires": {"xy": "2.0.0"}},
                "x_": 0,
            }),
        ),
        (
            "unknown property",
            json!({
                "test": {"requires": {"xy": "2.0.0"}},
                "foo": 0,
            }),
        ),
        // configure
        ("configure array", json!({"configure": ["2.0.0"]})),
        ("configure object", json!({"configure": {}})),
        ("configure string", json!({"configure": "2.0.0"})),
        ("configure bool", json!({"configure": true})),
        ("configure number", json!({"configure": 42})),
        ("configure null", json!({"configure": null})),
        // build
        ("build array", json!({"build": ["2.0.0"]})),
        ("build object", json!({"build": {}})),
        ("build string", json!({"build": "2.0.0"})),
        ("build bool", json!({"build": true})),
        ("build number", json!({"build": 42})),
        ("build null", json!({"build": null})),
        // test
        ("test array", json!({"test": ["2.0.0"]})),
        ("test object", json!({"test": {}})),
        ("test string", json!({"test": "2.0.0"})),
        ("test bool", json!({"test": true})),
        ("test number", json!({"test": 42})),
        ("test null", json!({"test": null})),
        // runtime
        ("runtime array", json!({"runtime": ["2.0.0"]})),
        ("runtime object", json!({"runtime": {}})),
        ("runtime string", json!({"runtime": "2.0.0"})),
        ("runtime bool", json!({"runtime": true})),
        ("runtime number", json!({"runtime": 42})),
        ("runtime null", json!({"runtime": null})),
        // develop
        ("develop array", json!({"develop": ["2.0.0"]})),
        ("develop object", json!({"develop": {}})),
        ("develop string", json!({"develop": "2.0.0"})),
        ("develop bool", json!({"develop": true})),
        ("develop number", json!({"develop": 42})),
        ("develop null", json!({"develop": null})),
    ] {
        if schemas.validate(&invalid_prereqs.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_prereqs.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_repository() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the repository schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "repository");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_repository in [
        ("web", json!({"web": "https://example.com"})),
        (
            "type and url",
            json!({"type": "git", "url": "https://example.com"}),
        ),
        (
            "x_ property",
            json!({"web": "https://example.com", "x_y": 0}),
        ),
        (
            "X_ property",
            json!({"web": "https://example.com", "X_y": 0}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_repository.1, idx) {
            panic!("extension {} failed: {e}", valid_repository.0);
        }
    }

    for invalid_repository in [
        ("empty array", json!([])),
        ("empty string", json!("")),
        ("empty string in array", json!(["hi", ""])),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        (
            "bare x_ property",
            json!({"web": "https://example.com", "x_": 0}),
        ),
        (
            "unknown property",
            json!({"web": "https://example.com", "foo": 0}),
        ),
        ("url without type", json!({"url": "x:y"})),
        ("type without url", json!({"type": "cvs"})),
        // web
        ("bad web URL", json!({"web": ":hello"})),
        ("web array", json!({"web": ["x:y"]})),
        ("web object", json!({"web": {}})),
        ("web bool", json!({"web": true})),
        ("web number", json!({"web": 42})),
        ("web null", json!({"web": null})),
        // url
        ("bad url", json!({"type": "git", "url": ":hello"})),
        ("url array", json!({"type": "git", "url": ["x:y"]})),
        ("url object", json!({"type": "git", "url": {}})),
        ("url bool", json!({"type": "git", "url": true})),
        ("url number", json!({"type": "git", "url": 42})),
        ("url null", json!({"type": "git", "url": null})),
        // type
        ("uppercase type", json!({"url": "x:y", "type": "FOO"})),
        ("mixed type", json!({"url": "x:y", "type": "Foo"})),
        ("empty type", json!({"url": "x:y", "type": ""})),
        ("null object", json!({"url": "x:y", "type": {}})),
        ("null array", json!({"url": "x:y", "type": ["git"]})),
        ("null number", json!({"url": "x:y", "type": 42})),
        ("null bool", json!({"url": "x:y", "type": true})),
    ] {
        if schemas.validate(&invalid_repository.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_repository.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_resources() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the resources schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "resources");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_resources in [
        ("homepage", json!({"homepage": "https://example.com"})),
        (
            "bugtracker web",
            json!({"bugtracker": {"web": "https://foo.com"}}),
        ),
        (
            "bugtracker mailto",
            json!({"bugtracker": {"mailto": "hi@example.com"}}),
        ),
        (
            "repository web",
            json!({"repository": {"web": "https://example.com"}}),
        ),
        (
            "repository url and type",
            json!({"repository": {"type": "git", "url": "https://example.com"}}),
        ),
        ("x_ property", json!({"homepage": "x:y", "x_y": 0})),
        ("X_ property", json!({"homepage": "x:y", "X_y": 0})),
    ] {
        if let Err(e) = schemas.validate(&valid_resources.1, idx) {
            panic!("extension {} failed: {e}", valid_resources.0);
        }
    }

    for invalid_resources in [
        ("empty array", json!([])),
        ("empty string", json!("")),
        ("empty string in array", json!(["hi", ""])),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("bare x_ property", json!({"homepage": "x:y", "x_": 0})),
        ("unknown property", json!({"homepage": "x:y", "foo": 0})),
        // homepage
        ("bad homepage url", json!({"homepage": ":hi"})),
        ("homepage array", json!({"homepage": ["x:y"]})),
        ("homepage object", json!({"homepage": {}})),
        ("homepage bool", json!({"homepage": true})),
        ("homepage number", json!({"homepage": 42})),
        ("homepage null", json!({"homepage": null})),
        // bugtracker
        (
            "bad bugtracker url",
            json!({"bugtracker": {"web": "3ttp://a.com"}}),
        ),
        ("bugtracker array", json!({"bugtracker": ["x:y"]})),
        ("bugtracker empty object", json!({"bugtracker": {}})),
        ("bugtracker bool", json!({"bugtracker": true})),
        ("bugtracker number", json!({"bugtracker": 42})),
        ("bugtracker null", json!({"bugtracker": null})),
        // repository
        (
            "bad repository url",
            json!({"repository": {"web": "3ttp://a.com"}}),
        ),
        ("repository array", json!({"repository": ["x:y"]})),
        ("repository empty object", json!({"repository": {}})),
        ("repository bool", json!({"repository": true})),
        ("repository number", json!({"repository": 42})),
        ("repository null", json!({"repository": null})),
    ] {
        if schemas.validate(&invalid_resources.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_resources.0)
        }
    }

    Ok(())
}

fn valid_distribution() -> Value {
    json!({
       "name": "pgTAP",
       "abstract": "Unit testing for PostgreSQL",
       "description": "pgTAP is a suite of database functions that make it easy to write TAP-emitting unit tests in psql scripts or xUnit-style test functions.",
       "version": "0.26.0",
       "maintainer": [
          "David E. Wheeler <david@justatheory.com>",
          "pgTAP List <pgtap-users@pgfoundry.org>"
       ],
       "license": {
          "PostgreSQL": "http://www.postgresql.org/about/licence"
       },
       "prereqs": {
          "runtime": {
             "requires": {
                "plpgsql": 0,
                "PostgreSQL": "8.0.0"
             },
             "recommends": {
                "PostgreSQL": "8.4.0"
             }
          }
       },
       "provides": {
         "pgtap": {
           "abstract": "Unit testing for PostgreSQL",
           "file": "pgtap.sql",
           "version": "0.26.0"
         }
       },
       "resources": {
          "homepage": "http://pgtap.org/",
          "bugtracker": {
             "web": "https://github.com/theory/pgtap/issues"
          },
          "repository": {
            "url":  "https://github.com/theory/pgtap.git",
            "web":  "https://github.com/theory/pgtap",
            "type": "git"
          }
       },
       "generated_by": "David E. Wheeler",
       "meta-spec": {
          "version": "1.0.0",
          "url": "https://pgxn.org/meta/spec.txt"
       },
       "tags": [
          "testing",
          "unit testing",
          "tap",
          "tddd",
          "test driven database development"
       ]
    })
}

#[test]
fn test_v1_distribution() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the distribution schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "distribution");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Make sure the valid distribution is in fact valid.
    let meta = valid_distribution();
    if let Err(e) = schemas.validate(&meta, idx) {
        panic!("valid_distribution meta failed: {e}");
    }

    // Cases ported from https://github.com/pgxn/pgxn-meta-validator/blob/v0.16.0/t/validator.t

    // type Checker = fn(&Value) -> bool;
    type Obj = Map<String, Value>;
    type Callback = fn(&mut Obj);

    static VALID_TEST_CASES: &[(&str, Callback)] = &[
        ("no change", |_: &mut Obj| {}),
        ("license apache_2_0", |m: &mut Obj| {
            m.insert("license".to_string(), json!("apache_2_0"));
        }),
        ("license postgresql", |m: &mut Obj| {
            m.insert("license".to_string(), json!("postgresql"));
        }),
        ("license array", |m: &mut Obj| {
            m.insert("license".to_string(), json!(["postgresql", "perl_5"]));
        }),
        ("license object", |m: &mut Obj| {
            m.insert("license".to_string(), json!({"foo": "https://example.com"}));
        }),
        ("provides docfile", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("docfile".to_string(), json!("foo/bar.txt"));
        }),
        ("provides no abstract", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.remove("abstract");
        }),
        ("provides custom key", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("x_foo".to_string(), json!(1));
        }),
        ("no spec URL", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.remove("url");
        }),
        ("spec custom key", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.insert("x_foo".to_string(), json!(1));
        }),
        ("multibyte name", |m: &mut Obj| {
            m.insert("name".to_string(), json!("yoŭ_know"));
        }),
        ("emoji name", |m: &mut Obj| {
            m.insert("name".to_string(), json!("📀📟🎱"));
        }),
        ("name with dash", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo-bar"));
        }),
        ("no generated_by", |m: &mut Obj| {
            m.remove("generated_by");
        }),
        ("one tag", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(["foo"]));
        }),
        ("no_index file", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": ["foo"]}));
        }),
        ("no_index file string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": "foo"}));
        }),
        ("no_index directory", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": ["foo"]}));
        }),
        ("no_index directory string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": "foo"}));
        }),
        ("no_index file and directory", |m: &mut Obj| {
            m.insert(
                "no_index".to_string(),
                json!({"file": ["foo", "bar"], "directory": "foo"}),
            );
        }),
        ("no_index custom key", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": "foo", "X_foo": 1}));
        }),
        // configure
        ("configure requires prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "configure".to_string(),
                json!({"requires": {"foo": "1.0.0"}}),
            );
        }),
        ("configure recommends prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "configure".to_string(),
                json!({"recommends": {"foo": "1.0.0"}}),
            );
        }),
        ("configure suggests prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "configure".to_string(),
                json!({"suggests": {"foo": "1.0.0"}}),
            );
        }),
        ("configure conflicts prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "configure".to_string(),
                json!({"conflicts": {"foo": "1.0.0"}}),
            );
        }),
        // build
        ("build requires prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("build".to_string(), json!({"requires": {"foo": "1.0.0"}}));
        }),
        ("build recommends prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("build".to_string(), json!({"recommends": {"foo": "1.0.0"}}));
        }),
        ("build suggests prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("build".to_string(), json!({"suggests": {"foo": "1.0.0"}}));
        }),
        ("build conflicts prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("build".to_string(), json!({"conflicts": {"foo": "1.0.0"}}));
        }),
        // test
        ("test requires prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("test".to_string(), json!({"requires": {"foo": "1.0.0"}}));
        }),
        ("test recommends prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("test".to_string(), json!({"recommends": {"foo": "1.0.0"}}));
        }),
        ("test suggests prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("test".to_string(), json!({"suggests": {"foo": "1.0.0"}}));
        }),
        ("test conflicts prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("test".to_string(), json!({"conflicts": {"foo": "1.0.0"}}));
        }),
        // runtime
        ("runtime requires prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("runtime".to_string(), json!({"requires": {"foo": "1.0.0"}}));
        }),
        ("runtime recommends prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"recommends": {"foo": "1.0.0"}}),
            );
        }),
        ("runtime suggests prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("runtime".to_string(), json!({"suggests": {"foo": "1.0.0"}}));
        }),
        ("runtime conflicts prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"conflicts": {"foo": "1.0.0"}}),
            );
        }),
        // develop
        ("develop requires prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("develop".to_string(), json!({"requires": {"foo": "1.0.0"}}));
        }),
        ("develop recommends prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "develop".to_string(),
                json!({"recommends": {"foo": "1.0.0"}}),
            );
        }),
        ("develop suggests prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("develop".to_string(), json!({"suggests": {"foo": "1.0.0"}}));
        }),
        ("develop conflicts prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "develop".to_string(),
                json!({"conflicts": {"foo": "1.0.0"}}),
            );
        }),
        // version range operators
        ("version range with == operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": "==1.0.0"}}),
            );
        }),
        ("version range with != operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": "!=1.0.0"}}),
            );
        }),
        ("version range with > operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": ">1.0.0"}}),
            );
        }),
        ("version range with < operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": "<1.0.0"}}),
            );
        }),
        ("version range with >= operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": ">=1.0.0"}}),
            );
        }),
        ("version range with <= operator", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": "<=1.0.0"}}),
            );
        }),
        ("prereq complex version range", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert(
                "runtime".to_string(),
                json!({"requires": {"foo": ">= 1.2.0, != 1.5.0, < 2.0.0"}}),
            );
        }),
        ("prereq version 0", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            prereqs.insert("runtime".to_string(), json!({"requires": {"foo": 0}}));
        }),
        // release_status
        ("no release_status", |m: &mut Obj| {
            m.remove("release_status");
        }),
        ("release_status stable", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!("stable"));
        }),
        ("release_status testing", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!("testing"));
        }),
        ("release_status unstable", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!("unstable"));
        }),
        // resources
        ("no resources", |m: &mut Obj| {
            m.remove("resources");
        }),
        ("homepage resource", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert("homepage".to_string(), json!("https://foo.com"));
        }),
        ("bugtracker resource", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "bugtracker".to_string(),
                json!({
                    "web": "https://example.com/",
                    "mailto": "foo@example.com",
                }),
            );
        }),
        ("bugtracker web", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "bugtracker".to_string(),
                json!({"web": "https://example.com/"}),
            );
        }),
        ("bugtracker mailto", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "bugtracker".to_string(),
                json!({"mailto": "foo@example.com"}),
            );
        }),
        ("bugtracker custom", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "bugtracker".to_string(),
                json!({"mailto": "foo@example.com", "x_foo": 1}),
            );
        }),
        ("repository resource", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "repository".to_string(),
                json!({
                    "web": "https://example.com",
                    "url": "git://example.com",
                    "type": "git",
                }),
            );
        }),
        ("repository resource url", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "repository".to_string(),
                json!({
                    "url": "git://example.com",
                    "type": "git",
                }),
            );
        }),
        ("repository resource web", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "repository".to_string(),
                json!({"web": "https://example.com"}),
            );
        }),
        ("repository custom", |m: &mut Obj| {
            let resources = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            resources.insert(
                "repository".to_string(),
                json!({"web": "https://example.com", "x_foo": 1}),
            );
        }),
    ];

    for tc in VALID_TEST_CASES {
        let mut meta = valid_distribution();
        let map = meta.as_object_mut().unwrap();
        tc.1(map);
        if let Err(e) = schemas.validate(&meta, idx) {
            panic!("distribution {} failed: {e}", tc.0);
        }
    }

    static INVALID_TEST_CASES: &[(&str, Callback)] = &[
        ("no name", |m: &mut Obj| {
            m.remove("name");
        }),
        ("no version", |m: &mut Obj| {
            m.remove("version");
        }),
        ("no abstract", |m: &mut Obj| {
            m.remove("abstract");
        }),
        ("no maintainer", |m: &mut Obj| {
            m.remove("maintainer");
        }),
        ("no license", |m: &mut Obj| {
            m.remove("license");
        }),
        ("no meta-spec", |m: &mut Obj| {
            m.remove("meta-spec");
        }),
        ("no provides", |m: &mut Obj| {
            m.remove("provides");
        }),
        ("bad version", |m: &mut Obj| {
            m.insert("version".to_string(), json!("1.0"));
        }),
        ("deprecated version", |m: &mut Obj| {
            m.insert("version".to_string(), json!("1.0.0v1"));
        }),
        ("version 0", |m: &mut Obj| {
            m.insert("version".to_string(), json!(0));
        }),
        ("provides version 0", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("version".to_string(), json!(0));
        }),
        ("bad provides version", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("version".to_string(), json!("hi"));
        }),
        ("bad prereq version", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!("1.2.0b1"));
        }),
        ("prereq null version", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!(null));
        }),
        ("prereq invalid version", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!("1.0"));
        }),
        ("prereq invalid version op", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!("= 1.0.0"));
        }),
        ("prereq wtf version op", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!("*** 1.0.0"));
        }),
        ("prereq wtf version leading comma", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            let requires = runtime
                .get_mut("requires")
                .unwrap()
                .as_object_mut()
                .unwrap();
            requires.insert("plpgsql".to_string(), json!(", 1.0.0"));
        }),
        ("invalid prereq phase", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            runtime.insert("genesis".to_string(), json!("1.0.0"));
        }),
        ("non-map prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            runtime.insert("requires".to_string(), json!(["PostgreSQL", "1.0.0"]));
        }),
        ("non-term prereq", |m: &mut Obj| {
            let prereqs = m.get_mut("prereqs").unwrap().as_object_mut().unwrap();
            let runtime = prereqs.get_mut("runtime").unwrap().as_object_mut().unwrap();
            runtime.insert("requires".to_string(), json!({"foo/bar": "1.0.0"}));
        }),
        ("invalid key", |m: &mut Obj| {
            m.insert("foo".to_string(), json!(1));
        }),
        ("invalid license", |m: &mut Obj| {
            m.insert("license".to_string(), json!("gobbledygook"));
        }),
        ("invalid licenses", |m: &mut Obj| {
            m.insert("license".to_string(), json!(["bsd", "gobbledygook"]));
        }),
        ("invalid license URL", |m: &mut Obj| {
            m.insert("license".to_string(), json!({"foo": ":not a url"}));
        }),
        ("no provides file", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.remove("file");
        }),
        ("no provides version", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.remove("version");
        }),
        ("provides array", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("version".to_string(), json!(["pgtap", "0.24.0"]));
        }),
        ("null provides file", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("file".to_string(), json!(null));
        }),
        ("null provides abstract", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("abstract".to_string(), json!(null));
        }),
        ("null provides version", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("abstract".to_string(), json!(null));
        }),
        ("null provides docfile", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("docfile".to_string(), json!(null));
        }),
        ("bad provides custom key", |m: &mut Obj| {
            let provides = m.get_mut("provides").unwrap().as_object_mut().unwrap();
            let pgtap = provides.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("w00t".to_string(), json!("hi"));
        }),
        ("alt spec version", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.insert("version".to_string(), json!("2.0.0"));
        }),
        ("no spec version", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.remove("version");
        }),
        ("bad spec URL", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.insert("url".to_string(), json!("not a url"));
        }),
        ("name with newline", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo\nbar"));
        }),
        ("name with return", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo\rbar"));
        }),
        ("name with slash", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo/bar"));
        }),
        ("name with backslash", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo\\\\bar"));
        }),
        ("name with space", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo bar"));
        }),
        ("short name", |m: &mut Obj| {
            m.insert("name".to_string(), json!("x"));
        }),
        ("null name", |m: &mut Obj| {
            m.insert("name".to_string(), json!(null));
        }),
        ("array name", |m: &mut Obj| {
            m.insert("name".to_string(), json!([]));
        }),
        ("object name", |m: &mut Obj| {
            m.insert("name".to_string(), json!({}));
        }),
        ("bool name", |m: &mut Obj| {
            m.insert("name".to_string(), json!(false));
        }),
        ("number name", |m: &mut Obj| {
            m.insert("name".to_string(), json!(42));
        }),
        ("empty description", |m: &mut Obj| {
            m.insert("description".to_string(), json!(""));
        }),
        ("null description", |m: &mut Obj| {
            m.insert("description".to_string(), json!(null));
        }),
        ("array description", |m: &mut Obj| {
            m.insert("description".to_string(), json!([]));
        }),
        ("object description", |m: &mut Obj| {
            m.insert("description".to_string(), json!({}));
        }),
        ("bool description", |m: &mut Obj| {
            m.insert("description".to_string(), json!(false));
        }),
        ("number description", |m: &mut Obj| {
            m.insert("description".to_string(), json!(42));
        }),
        ("array generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!([]));
        }),
        ("object generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!({}));
        }),
        ("bool generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!(false));
        }),
        ("number generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!(42));
        }),
        ("null generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!(null));
        }),
        ("null generated_by", |m: &mut Obj| {
            m.insert("generated_by".to_string(), json!(null));
        }),
        ("null tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(null));
        }),
        ("empty tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!([]));
        }),
        ("object tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!({}));
        }),
        ("bool tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(false));
        }),
        ("number tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(42));
        }),
        ("null tag", |m: &mut Obj| {
            m.insert("tags".to_string(), json!([null]));
        }),
        ("empty tag", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(["", "foo"]));
        }),
        ("long tag", |m: &mut Obj| {
            m.insert("tags".to_string(), json!(["x".repeat(256)]));
        }),
        ("object tag", |m: &mut Obj| {
            m.insert("tags".to_string(), json!([{}]));
        }),
        ("bool tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!([false]));
        }),
        ("number tags", |m: &mut Obj| {
            m.insert("tags".to_string(), json!([42]));
        }),
        ("no_index empty file string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": ""}));
        }),
        ("no_index null file string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": null}));
        }),
        ("no_index null file empty array", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": []}));
        }),
        ("no_index null file object", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": {}}));
        }),
        ("no_index null file bool", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": true}));
        }),
        ("no_index null file number", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": 42}));
        }),
        ("no_index empty file array string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [""]}));
        }),
        ("no_index undef file array string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [null]}));
        }),
        ("no_index undef file array number", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [42]}));
        }),
        ("no_index undef file array bool", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [true]}));
        }),
        ("no_index undef file array obj", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [{}]}));
        }),
        ("no_index undef file array array", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"file": [[]]}));
        }),
        ("no_index empty directory string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": ""}));
        }),
        ("no_index null directory string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": null}));
        }),
        ("no_index null directory empty array", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": []}));
        }),
        ("no_index null directory object", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": {}}));
        }),
        ("no_index null directory bool", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": true}));
        }),
        ("no_index null directory number", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": 42}));
        }),
        ("no_index empty directory array string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [""]}));
        }),
        ("no_index undef directory array string", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [null]}));
        }),
        ("no_index undef directory array number", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [42]}));
        }),
        ("no_index undef directory array bool", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [true]}));
        }),
        ("no_index undef directory array obj", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [{}]}));
        }),
        ("no_index undef directory array array", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"directory": [[]]}));
        }),
        ("no_index bad key", |m: &mut Obj| {
            m.insert("no_index".to_string(), json!({"foo": "hi"}));
        }),
        ("invalid release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!("rocking"));
        }),
        ("null release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!(null));
        }),
        ("bool release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!(true));
        }),
        ("number release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!(42));
        }),
        ("object release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!({}));
        }),
        ("array release_status", |m: &mut Obj| {
            m.insert("release_status".to_string(), json!([]));
        }),
        ("null resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!(null));
        }),
        ("bool resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!(true));
        }),
        ("number resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!(42));
        }),
        ("object resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!({}));
        }),
        ("array resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!([]));
        }),
        ("homepage resource null", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("homepage".to_string(), json!(null));
        }),
        ("homepage resource non-url", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("homepage".to_string(), json!("not a url"));
        }),
        ("bugtracker resource null", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!(null));
        }),
        ("bugtracker resource array", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!(["hi"]));
        }),
        ("bugtracker resource invalid key", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!({"foo": 1}));
        }),
        ("bugtracker resource array", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!(["hi"]));
        }),
        ("bugtracker invalid url", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!({"web": "not a url"}));
        }),
        ("bugtracker invalid email", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("bugtracker".to_string(), json!({"mailto": "not an email"}));
        }),
        ("repository resource undef", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("repository".to_string(), json!(null));
        }),
        ("repository resource array", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("repository".to_string(), json!(["hi"]));
        }),
        ("repository resource invalid key", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("repository".to_string(), json!({"foo": 1}));
        }),
        ("repository resource invalid url", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert(
                "repository".to_string(),
                json!({"url": "not a url", "type": "x"}),
            );
        }),
        ("repository resource invalid web url", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert("repository".to_string(), json!({"web": "not a url"}));
        }),
        ("repository resource invalid type", |m: &mut Obj| {
            let res = m.get_mut("resources").unwrap().as_object_mut().unwrap();
            res.insert(
                "repository".to_string(),
                json!({"url": "x:y", "type": "Foo"}),
            );
        }),
    ];
    for tc in INVALID_TEST_CASES {
        let mut meta = valid_distribution();
        let map = meta.as_object_mut().unwrap();
        tc.1(map);
        if schemas.validate(&meta, idx).is_ok() {
            panic!("{} unexpectedly passed!", tc.0)
        }
    }

    Ok(())
}

#[test]
fn test_v1_release() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the distribution schema.
    let mut compiler = new_compiler("schema/v1")?;
    let mut schemas = Schemas::new();
    let release_id = id_for(SCHEMA_VERSION, "release");
    let release_idx = compiler.compile(&release_id, &mut schemas)?;
    let dist_id = id_for(SCHEMA_VERSION, "distribution");
    let dist_idx = compiler.compile(&dist_id, &mut schemas)?;

    // Now try it with various release metadata.
    for (name, release_meta) in [
        (
            "all release fields",
            json!({
              "user": "theory",
              "date": "2024-09-12T20:39:11Z",
              "sha1": "d833511c7ebb9c1875426ca8a93edcacd0787c46",
            }),
        ),
        (
            "different release fields",
            json!({
              "user": "okbob",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b",
            }),
        ),
    ] {
        // Merge the release metadata; the release schema should validate it.
        let mut meta = valid_distribution();
        json_patch::merge(&mut meta, &release_meta);
        if let Err(e) = schemas.validate(&meta, release_idx) {
            panic!("{name} with release meta failed: {e}");
        }

        // But it should fail on just distribution metadata.
        if schemas.validate(&meta, dist_idx).is_ok() {
            panic!("{name} unexpectedly validated by distribution schema");
        }
    }

    // Now try invalid cases.
    for (name, release_meta, err) in [
        (
            "no release_fields",
            json!({}),
            "missing properties 'user', 'date', 'sha1'",
        ),
        (
            "missing user",
            json!({
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "missing properties 'user'",
        ),
        (
            "user empty",
            json!({
              "user": "",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': length must be >=2, but got 0",
        ),
        (
            "user too short",
            json!({
              "user": "x",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': length must be >=2, but got 1",
        ),
        (
            "user number",
            json!({
              "user": 42,
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': want string, but got number",
        ),
        (
            "user bool",
            json!({
              "user": true,
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': want string, but got boolean",
        ),
        (
            "user null",
            json!({
              "user": null,
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "missing properties 'user'",
        ),
        (
            "user array",
            json!({
              "user": ["hi"],
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': want string, but got array",
        ),
        (
            "user object",
            json!({
              "user": {"hi": 42},
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/user': want string, but got object",
        ),
        (
            "missing date",
            json!({
                "user": "hi",
                "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "missing properties 'date'",
        ),
        (
            "invalid date",
            json!({
              "user": "hi",
              "date": "2019-09-23T27:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': '2019-09-23T27:16:45Z' is not valid date-time",
        ),
        (
            "date empty",
            json!({
              "user": "hi",
              "date": "",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': '' is not valid date-time",
        ),
        (
            "date number",
            json!({
              "user": "hi",
              "date": 98.6,
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': want string, but got number",
        ),
        (
            "date bool",
            json!({
              "user": "hi",
              "date": false,
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': want string, but got boolean",
        ),
        (
            "date null",
            json!({
              "user": "hi",
              "date": null,
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "missing properties 'date'",
        ),
        (
            "date array",
            json!({
              "user": "hi",
              "date": ["2024-09-12T20:39:11Z"],
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': want string, but got array",
        ),
        (
            "date array",
            json!({
              "user": "hi",
              "date": {"x": "2024-09-12T20:39:11Z"},
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b"
            }),
            "'/date': want string, but got object",
        ),
        (
            "missing sha1",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
            }),
            "missing properties 'sha1'",
        ),
        (
            "null sha1",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": null,
            }),
            "missing properties 'sha1'",
        ),
        (
            "empty sha1",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": "",
            }),
            "'/sha1': '' does not match pattern",
        ),
        (
            "short sha1",
            json!({
              "user": "hi",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8"
            }),
            "'/sha1': '0389be689af6992b4da520ec510d147bae411e8' does not match pattern",
        ),
        (
            "long sha1",
            json!({
              "user": "hi",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b1"
            }),
            "'/sha1': '0389be689af6992b4da520ec510d147bae411e8b1' does not match pattern",
        ),
        (
            "invalid sha1 hex",
            json!({
              "user": "hi",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8g"
            }),
            "'/sha1': '0389be689af6992b4da520ec510d147bae411e8g' does not match pattern",
        ),
        (
            "sha1 bool",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": true,
            }),
            "'/sha1': want string, but got boolean",
        ),
        (
            "sha1 number",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": 42,
            }),
            "'/sha1': want string, but got number",
        ),
        (
            "sha1 array",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": ["0389be689af6992b4da520ec510d147bae411e8b"],
            }),
            "'/sha1': want string, but got array",
        ),
        (
            "sha1 object",
            json!({
                "user": "hi",
                "date": "2019-09-23T17:16:45Z",
                "sha1": {"0389be689af6992b4da520ec510d147bae411e8b": true},
            }),
            "'/sha1': want string, but got object",
        ),
        (
            "missing required distribution field",
            json!({
              "user": "okbob",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b",
              "version": null,
            }),
            "missing properties 'version'",
        ),
    ] {
        // Merge the release metadata; the release schema should validate it.
        let mut meta = valid_distribution();
        json_patch::merge(&mut meta, &release_meta);
        match schemas.validate(&meta, release_idx) {
            Err(e) => assert!(e.to_string().contains(err), "{name} Error: {e}"),
            Ok(_) => panic!("{name} unexpectedly succeeded"),
        }
    }

    Ok(())
}
