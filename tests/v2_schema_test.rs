use std::error::Error;

use boon::Schemas;
use serde_json::{json, Map, Value};

// importing common module.
mod common;
use common::*;

const SCHEMA_VERSION: u8 = 2;

#[test]
fn test_schema_v2() -> Result<(), Box<dyn Error>> {
    test_schema_version(SCHEMA_VERSION)
}

#[test]
fn test_v2_term() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the term schema.
    let compiler = new_compiler("schema/v2")?;
    test_term_schema(compiler, SCHEMA_VERSION)
}

#[test]
fn test_v2_tags() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the tags schema.
    let compiler = new_compiler("schema/v2")?;
    test_tags_schema(compiler, SCHEMA_VERSION)
}

#[test]
fn test_v2_semver() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "semver");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_version in VALID_SEMVERS {
        let vv = json!(valid_version);
        if valid_version.contains('+') {
            // Metadata is forbidden in v2 semvers.
            if schemas.validate(&vv, idx).is_ok() {
                panic!("{} unexpectedly passed!", valid_version)
            }
        } else if let Err(e) = schemas.validate(&vv, idx) {
            panic!("{} failed: {e}", valid_version);
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
fn test_v2_path() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "path");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid paths.
    for valid in [
        json!("README.txt"),
        json!(".git"),
        json!("src/pair.c"),
        json!(".github/workflows/"),
        json!("this\\\\and\\\\that.txt"),
    ] {
        if let Err(e) = schemas.validate(&valid, idx) {
            panic!("{} failed: {e}", valid);
        }
    }

    // Test invalid paths.
    for invalid in [
        json!("\\foo.md"),
        json!("this\\and\\that.txt"),
        json!("/absolute/path"),
        // Enforced only by custom format for now.
        // https://github.com/santhosh-tekuri/boon/issues/19
        json!("../outside/path"),
        json!("thing/../other"),
        json!(null),
        json!(""),
        json!("C:\\foo"),
    ] {
        if schemas.validate(&invalid, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid)
        }
    }
    Ok(())
}

#[test]
fn test_v2_glob() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "glob");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid globs.
    for valid in [
        json!("README.txt"),
        json!("/.git"),
        json!("/src/pair.c"),
        json!("/src/private.*"),
        json!("*.html"),
        json!("*.?tml"),
        json!("foo.?tml"),
        json!("[xX]_*.*"),
        json!("[a-z]*.txt"),
        json!("this\\\\and\\\\that.txt"),
    ] {
        if let Err(e) = schemas.validate(&valid, idx) {
            panic!("{} failed: {e}", valid);
        }
    }

    // Test invalid globs.
    for invalid in [
        json!("this\\and\\that.txt"),
        json!(null),
        json!(""),
        json!("C:\\foo"),
    ] {
        if schemas.validate(&invalid, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid)
        }
    }

    Ok(())
}

#[test]
fn test_v2_version_range() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the version_range schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "version_range");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_version in [VALID_SEMVERS, &["1", "3", "2.1", "3.14"]].concat() {
        for op in ["", "==", "!=", ">", "<", ">=", "<="] {
            for append in [
                "",
                ",<= 1.1.2+meta",
                ",>= 2.0.0, 1.5.6",
                ",>= 2.0, 1.5",
                ",>= 2, ==1",
                ", >1.2.0, != 12.0.0, < 19.2.0",
            ] {
                let range = json!(format!("{}{}{}", op, valid_version, append));
                if let Err(e) = schemas.validate(&range, idx) {
                    panic!("{} failed: {e}", range);
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
        panic!("{} failed: {e}", zero);
    }

    // But version 0 cannot appear with any range operator or in any range.
    for op in ["", "==", "!=", ">", "<", ">=", "<="] {
        let range = json!(format!("{op}0"));
        if let Err(e) = schemas.validate(&range, idx) {
            panic!("{} failed: {e}", range);
        }
    }

    // Test invalid ranges.
    for invalid_range in [
        json!("x.y.z"),
        json!(null),
        json!(""),
        json!(">2.0 and <3.0"),
        json!("==2.0 or ==3.0"),
    ] {
        if schemas.validate(&invalid_range, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_range)
        }
    }

    Ok(())
}

#[test]
fn test_v2_license() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "license");
    let idx = compiler.compile(&id, &mut schemas)?;

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
        if let Err(e) = schemas.validate(&valid_license, idx) {
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
        if schemas.validate(&invalid_license, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_license)
        }
    }
    Ok(())
}

#[test]
fn test_v2_purl() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "purl");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid relative purls.
    for valid_purl in [
        json!("pkg:pgxn/pgtap"),
        json!("pkg:postgres/pg_regress"),
        json!("pkg:generic/python3"),
        json!("pkg:pypi/pyarrow@11.0.0"),
        json!("pkg:type/namespace/name"),
        json!("pkg:type/namespace/name@version"),
        json!("pkg:type/namespace/name@version?qualifiers"),
        json!("pkg:type/namespace/name@version?qualifiers#subpath"),
    ] {
        if let Err(e) = schemas.validate(&valid_purl, idx) {
            panic!("{} failed: {e}", valid_purl);
        }
    }

    // Test invalid purls.
    for invalid_purl in [
        json!("http://example.com"),
        json!("https://example.com"),
        json!("mailto:hi@example.com"),
        json!(null),
        json!("0"),
        json!(0),
        json!("\n\t"),
        json!("()"),
        json!("AND"),
        json!("OR"),
    ] {
        if schemas.validate(&invalid_purl, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_purl)
        }
    }
    Ok(())
}

#[test]
fn test_v2_platform() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "platform");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid relative platforms.
    for os in [
        "any",
        "aix",
        "android",
        "darwin",
        "dragonfly",
        "freebsd",
        "illumos",
        "ios",
        "js",
        "linux",
        "netbsd",
        "openbsd",
        "plan9",
        "solaris",
        "wasip1",
        "windows",
    ] {
        let platform = json!(os);
        if let Err(e) = schemas.validate(&platform, idx) {
            panic!("path pattern {} failed: {e}", platform);
        }

        let architectures = [
            "386", "amd64", "arm", "arm64", "loong64", "mips", "mips64", "mips64le", "mipsle",
            "ppc64", "ppc64le", "riscv64", "s390x", "sparc64", "wasm",
        ];

        for arch in architectures {
            let platform = json!(format!("{os}-{arch}"));
            if let Err(e) = schemas.validate(&platform, idx) {
                panic!("path pattern {} failed: {e}", platform);
            }
        }

        for version in [
            VALID_SEMVERS,
            &["1.0", "3.2.5", "2.1+beta", "3.14", "16.beta1", "17.+foo"],
        ]
        .concat()
        {
            if version.contains('-') {
                continue;
            }
            let platform = json!(format!("{os}-{version}"));
            if let Err(e) = schemas.validate(&platform, idx) {
                panic!("path pattern {} failed: {e}", platform);
            }

            for arch in architectures {
                let platform = json!(format!("{os}-{version}-{arch}"));
                if let Err(e) = schemas.validate(&platform, idx) {
                    panic!("path pattern {} failed: {e}", platform);
                }
            }
        }
    }

    // Test invalid platforms.
    for invalid_platform in [
        json!("darwin amd64"),
        json!("linux/amd64"),
        json!("x86_64"),
        json!("darwin_23.5.0_arm64"),
        json!(null),
        json!("0"),
        json!(0),
        json!("\n\t"),
        json!("()"),
    ] {
        if schemas.validate(&invalid_platform, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_platform)
        }
    }
    Ok(())
}

#[test]
fn test_v2_platforms() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "platforms");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid platforms.
    for valid in [
        ("two", json!(["darwin", "linux"])),
        ("three", json!(["darwin", "linux", "windows"])),
        ("versions", json!(["musllinux-2.5", "gnulinux-3.3"])),
        (
            "architectures",
            json!(["musllinux-amd64", "gnulinux-amd64"]),
        ),
        ("full", json!(["musllinux-2.5-amd64", "gnulinux-3.3-amd64"])),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    // Test invalid platforms.
    for invalid in [
        json!(["darwin amd64"]),
        json!(["linux/amd64"]),
        json!(["x86_64"]),
        json!(["darwin_23.5.0_arm64"]),
        json!([]),
        json!([null]),
        json!(["0"]),
        json!([0]),
        json!({}),
        json!(true),
        json!(42),
        json!(["\n\t"]),
        json!(["()"]),
        json!(["darwin", "x86_64"]),
    ] {
        if schemas.validate(&invalid, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid)
        }
    }
    Ok(())
}

#[test]
fn test_v2_maintainers() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "maintainers");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_maintainer in [
        (
            "min name length",
            json!([{"name": "x", "email": "x@example.com"}]),
        ),
        (
            "name and email",
            json!([{"name": "David E. Wheeler", "email": "theory@pgxn.org"}]),
        ),
        (
            "name and URL",
            json!([{"name": "David E. Wheeler", "url": "https://pgxn.org/user/theory"}]),
        ),
        (
            "two names and emails",
            json!([
                {"name": "Josh Berkus", "email": "jberkus@pgxn.org"},
            ]),
        ),
        (
            "two names and URLs",
            json!([
                {"name": "Josh Berkus", "url": "https://pgxn.org/user/jberkus"},
                {"name": "David E. Wheeler", "url": "https://pgxn.org/user/theory"},
            ]),
        ),
        (
            "all fields",
            json!([{
                "name": "David E. Wheeler",
                "email": "theory@pgxn.org",
                "url": "https://pgxn.org/user/theory",
            }]),
        ),
        (
            "multiple all fields",
            json!([
                {
                    "name": "David E. Wheeler",
                    "email": "theory@pgxn.org",
                    "url": "https://pgxn.org/user/theory",
                },
                {
                    "name": "Josh Berkus",
                    "email": "jberkus@pgxn.org",
                    "url": "https://pgxn.org/user/jberkus",
                },
            ]),
        ),
        (
            "custom x_",
            json!([{"name": "x", "email": "x@example.com", "x_y": true}]),
        ),
        (
            "custom X_",
            json!([{"name": "x", "email": "x@example.com", "X_z": true}]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_maintainer.1, idx) {
            panic!("{} failed: {e}", valid_maintainer.0);
        }
    }

    for invalid_maintainer in [
        ("empty array", json!([])),
        ("empty string", json!("")),
        ("string in array", json!(["hi", ""])),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("true in array", json!([true])),
        ("false in array", json!([false])),
        ("null in array", json!([null])),
        ("empty object", json!([{}])),
        ("name only", json!([{"name": "hi"}])),
        ("email only", json!([{"email": "hi@x.com"}])),
        ("url only", json!([{"url": "x:y"}])),
        (
            "url and email only",
            json!([{"url": "x:y", "email": "hi@x.com"}]),
        ),
        (
            "dupe",
            json!([
                {"name": "x", "email": "x@example.com"},
                {"name": "x", "email": "x@example.com"},
            ]),
        ),
        // Name
        ("empty name", json!([{"name": "", "url": "x:y"}])),
        ("null name", json!([{"name": null, "url": "x:y"}])),
        ("bool name", json!([{"name": true, "url": "x:y"}])),
        ("number name", json!([{"name": 42, "url": "x:y"}])),
        ("array name", json!([{"name": [], "url": "x:y"}])),
        ("object name", json!([{"name": {}, "url": "x:y"}])),
        // Email:
        ("invalid email", json!([{"name": "hi", "email": "not"}])),
        ("empty email", json!([{"name": "hi", "email": ""}])),
        ("null email", json!([{"name": "hi", "email": null}])),
        ("bool email", json!([{"name": "hi", "email": false}])),
        ("number email", json!([{"name": "hi", "email": 42}])),
        ("array email", json!([{"name": "hi", "email": []}])),
        ("object email", json!([{"name": "hi", "email": {}}])),
        // URL
        ("invalid url", json!([{"name": "hi", "url": "not a url"}])),
        ("empty url", json!([{"name": "hi", "url": ""}])),
        ("null url", json!([{"name": "hi", "url": null}])),
        ("bool url", json!([{"name": "hi", "url": false}])),
        ("number url", json!([{"name": "hi", "url": 42}])),
        ("array url", json!([{"name": "hi", "url": []}])),
        ("object url", json!([{"name": "hi", "url": {}}])),
        // Custom
        (
            "bare X_",
            json!([{"name": "x", "email": "x@example.com", "X_": true}]),
        ),
        (
            "bare x_",
            json!([{"name": "x", "email": "x@example.com", "x_": true}]),
        ),
    ] {
        if schemas.validate(&invalid_maintainer.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_maintainer.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_extension() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the extension schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "extension");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_extension in [
        (
            "required fields",
            json!({
                "sql": "widget.sql",
                "control": "widget.control",
            }),
        ),
        (
            "with abstract",
            json!({
                "sql": "widget.sql",
                "control": "widget.control",
                "abstract": "This and that",
            }),
        ),
        (
            "all fields",
            json!({
                "sql": "widget.sql",
                "control": "widget.control",
                "doc": "foo/bar.txt",
                "abstract": "This and that",
                "tle": true,
            }),
        ),
        (
            "x field",
            json!({
                "sql": "widget.sql",
                "control": "widget.control",
                "x_hi": true,
            }),
        ),
        (
            "X field",
            json!({
                "sql": "widget.sql",
                "control": "widget.control",
                "X_bar": 42,
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_extension.1, idx) {
            panic!("{} failed: {e}", valid_extension.0);
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
            json!({"sql": "widget.sql", "control": "x.control", "foo": "hi", }),
        ),
        (
            "bare x_",
            json!({ "sql": "widget.sql", "control": "x.control", "x_": "hi" }),
        ),
        (
            "bare X_",
            json!({ "sql": "widget.sql", "control": "x.control", "X_": "hi" }),
        ),
        // sql
        ("no sql", json!({"control": "x.control"})),
        ("null sql", json!({"sql": null, "control": "x.control"})),
        ("empty sql", json!({"sql": "", "control": "x.control"})),
        ("number sql", json!({"sql": 42, "control": "x.control"})),
        ("bool sql", json!({"sql": true, "control": "x.control"})),
        ("array sql", json!({"sql": [], "control": "x.control"})),
        ("object sql", json!({"sql": {}, "control": "x.control"})),
        // control
        ("no control", json!({"sql": "x.sql"})),
        ("null control", json!({"control": null, "sql": "x.sql"})),
        ("empty control", json!({"control": "", "sql": "x.sql"})),
        ("number control", json!({"control": 42, "sql": "x.sql"})),
        ("bool control", json!({"control": true, "sql": "x.sql"})),
        ("array control", json!({"control": [], "sql": "x.sql"})),
        ("object control", json!({"control": {}, "sql": "x.sql"})),
        // doc
        (
            "empty doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": ""}),
        ),
        (
            "null doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": null}),
        ),
        (
            "number doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": 42}),
        ),
        (
            "bool doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": true}),
        ),
        (
            "array doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": ["hi"]}),
        ),
        (
            "object doc",
            json!({"sql": "widget.sql", "control": "widget.control", "doc": {}}),
        ),
        // abstract
        (
            "empty abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": ""}),
        ),
        (
            "null abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": null}),
        ),
        (
            "number abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": 42}),
        ),
        (
            "bool abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": true}),
        ),
        (
            "array abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": ["hi"]}),
        ),
        (
            "object abstract",
            json!({"sql": "widget.sql", "control": "widget.control", "abstract": {}}),
        ),
        // tle
        (
            "empty tle",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": ""}),
        ),
        (
            "tle string",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": "true"}),
        ),
        (
            "null tle",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": null}),
        ),
        (
            "number tle",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": 42}),
        ),
        (
            "array tle",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": ["hi"]}),
        ),
        (
            "object tle",
            json!({"sql": "widget.sql", "control": "widget.control", "tle": {}}),
        ),
    ] {
        if schemas.validate(&invalid_extension.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_extension.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_module() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the extension schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "module");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_module in [
        ("hook", json!({"type": "hook", "lib": "src/hook"})),
        ("bgw", json!({"type": "bgw", "lib": "src/bgw"})),
        ("extension", json!({"type": "extension", "lib": "src/ext"})),
        (
            "with abstract",
            json!({"type": "hook", "lib": "src/hook", "abstract": "This and that"}),
        ),
        (
            "server",
            json!({"type": "hook", "lib": "src/hook", "preload": "server"}),
        ),
        (
            "session",
            json!({"type": "hook", "lib": "src/hook", "preload": "session"}),
        ),
        (
            "all fields",
            json!({"type": "hook", "lib": "src/hook", "doc": "bog.md", "abstract": "This and that", "preload": "session"}),
        ),
        (
            "x field",
            json!({"type": "hook", "lib": "src/hook", "x_hi": true}),
        ),
        (
            "X field",
            json!({"type": "hook", "lib": "src/hook", "X_bar": 42}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_module.1, idx) {
            panic!("{} failed: {e}", valid_module.0);
        }
    }

    for invalid_module in [
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
            json!({"type": "bgw", "lib": "src/bgw", "foo": "hi"}),
        ),
        (
            "bare x_",
            json!({ "type": "bgw", "lib": "src/bgw", "x_": "hi"}),
        ),
        (
            "bare X_",
            json!({ "type": "bgw", "lib": "src/bgw", "X_": "hi"}),
        ),
        // type
        ("no type", json!({"lib": "bgw"})),
        ("empty type", json!({"type": "", "lib": "bgw"})),
        ("invalid type", json!({"type": "burp", "lib": "bgw"})),
        ("null type", json!({"type": null, "lib": "bgw"})),
        ("empty type", json!({"type": "", "lib": "bgw"})),
        ("number type", json!({"type": 42, "lib": "bgw"})),
        ("bool type", json!({"type": true, "lib": "bgw"})),
        ("array type", json!({"type": [], "lib": "bgw"})),
        ("object type", json!({"type": {}, "lib": "bgw"})),
        // lib
        ("no lib", json!({"type": "bgw"})),
        ("null lib", json!({"lib": null, "type": "bgw"})),
        ("empty lib", json!({"lib": "", "type": "bgw"})),
        ("number lib", json!({"lib": 42, "type": "bgw"})),
        ("bool lib", json!({"lib": true, "type": "bgw"})),
        ("array lib", json!({"lib": [], "type": "bgw"})),
        ("object lib", json!({"lib": {}, "type": "bgw"})),
        // doc
        (
            "empty doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": ""}),
        ),
        (
            "null doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": null}),
        ),
        (
            "number doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": 42}),
        ),
        (
            "bool doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": true}),
        ),
        (
            "array doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": ["hi"]}),
        ),
        (
            "object doc",
            json!({"type": "bgw", "lib": "src/bgw", "doc": {}}),
        ),
        // abstract
        (
            "empty abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": ""}),
        ),
        (
            "null abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": null}),
        ),
        (
            "number abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": 42}),
        ),
        (
            "bool abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": true}),
        ),
        (
            "array abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": ["hi"]}),
        ),
        (
            "object abstract",
            json!({"type": "bgw", "lib": "src/bgw", "abstract": {}}),
        ),
        // preload
        (
            "empty preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": ""}),
        ),
        (
            "invalid preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": "startup"}),
        ),
        (
            "null preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": null}),
        ),
        (
            "number preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": 42}),
        ),
        (
            "bool preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": true}),
        ),
        (
            "array preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": ["hi"]}),
        ),
        (
            "object preload",
            json!({"type": "bgw", "lib": "src/bgw", "preload": {}}),
        ),
    ] {
        if schemas.validate(&invalid_module.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_module.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_app() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the extension schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "app");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_app in [
        ("bin only", json!({"bin": "bog"})),
        ("bin lang", json!({"bin": "bog", "lang": "perl"})),
        ("short lang", json!({"bin": "bog", "lang": "sh"})),
        ("doc", json!({"bin": "bog", "doc": "hi.md"})),
        ("lib", json!({"bin": "bog", "lib": "lib"})),
        ("man", json!({"bin": "bog", "man": "man"})),
        ("html", json!({"bin": "bog", "html": "html"})),
        (
            "abstract",
            json!({"bin": "bog", "abstract": "This and that"}),
        ),
        (
            "all fields",
            json!({
                "bin": "bog",
                "doc": "bog.md",
                "abstract": "This and that",
                "lib": "lib",
                "man": "man",
                "html": "html",
            }),
        ),
        ("x field", json!({"bin": "bog", "x_hi": true})),
        ("X field", json!({"bin": "bog", "X_bar": 42})),
    ] {
        if let Err(e) = schemas.validate(&valid_app.1, idx) {
            panic!("{} failed: {e}", valid_app.0);
        }
    }

    for invalid_app in [
        // Basics
        ("array", json!([])),
        ("string", json!("crank")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("invalid field", json!({"bin": "bog", "foo": "hi", })),
        ("bare x_", json!({ "bin": "bog", "x_": "hi" })),
        ("bare X_", json!({ "bin": "bog", "X_": "hi" })),
        // bin
        ("no bin", json!({"src": "x.src"})),
        ("null bin", json!({"bin": null, "src": "x.src"})),
        ("empty bin", json!({"bin": "", "src": "x.src"})),
        ("number bin", json!({"bin": 42, "src": "x.src"})),
        ("bool bin", json!({"bin": true, "src": "x.src"})),
        ("array bin", json!({"bin": [], "src": "x.src"})),
        ("object bin", json!({"bin": {}, "src": "x.src"})),
        // doc
        ("empty doc", json!({"bin": "bog", "doc": ""})),
        ("null doc", json!({"bin": "bog", "doc": null})),
        ("windows doc", json!({"bin": "bog", "doc": "c:\\foo"})),
        ("number doc", json!({"bin": "bog", "doc": 42})),
        ("bool doc", json!({"bin": "bog", "doc": true})),
        ("array doc", json!({"bin": "bog", "doc": ["hi"]})),
        ("object doc", json!({"bin": "bog", "doc": {}})),
        // lang
        ("empty lang", json!({"bin": "bog", "lang": ""})),
        ("null lang", json!({"bin": "bog", "lang": null})),
        ("number lang", json!({"bin": "bog", "lang": 42})),
        ("bool lang", json!({"bin": "bog", "lang": true})),
        ("array lang", json!({"bin": "bog", "lang": ["hi"]})),
        ("object lang", json!({"bin": "bog", "lang": {}})),
        // abstract
        ("empty abstract", json!({"bin": "bog", "abstract": ""})),
        ("null abstract", json!({"bin": "bog", "abstract": null})),
        ("number abstract", json!({"bin": "bog", "abstract": 42})),
        ("bool abstract", json!({"bin": "bog", "abstract": true})),
        ("array abstract", json!({"bin": "bog", "abstract": ["hi"]})),
        ("object abstract", json!({"bin": "bog", "abstract": {}})),
        // lib
        ("empty lib", json!({"bin": "bog", "lib": ""})),
        ("null lib", json!({"bin": "bog", "lib": null})),
        ("windows lib", json!({"bin": "bog", "lib": "c:\\foo"})),
        ("number lib", json!({"bin": "bog", "lib": 42})),
        ("bool lib", json!({"bin": "bog", "lib": true})),
        ("array lib", json!({"bin": "bog", "lib": ["hi"]})),
        ("object lib", json!({"bin": "bog", "lib": {}})),
        // man
        ("empty man", json!({"bin": "bog", "man": ""})),
        ("null man", json!({"bin": "bog", "man": null})),
        ("windows man", json!({"bin": "bog", "man": "c:\\foo"})),
        ("number man", json!({"bin": "bog", "man": 42})),
        ("bool man", json!({"bin": "bog", "man": true})),
        ("array man", json!({"bin": "bog", "man": ["hi"]})),
        ("object man", json!({"bin": "bog", "man": {}})),
        // html
        ("empty html", json!({"bin": "bog", "html": ""})),
        ("null html", json!({"bin": "bog", "html": null})),
        ("windows html", json!({"bin": "bog", "html": "c:\\foo"})),
        ("number html", json!({"bin": "bog", "html": 42})),
        ("bool html", json!({"bin": "bog", "html": true})),
        ("array html", json!({"bin": "bog", "html": ["hi"]})),
        ("object html", json!({"bin": "bog", "html": {}})),
    ] {
        if schemas.validate(&invalid_app.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_app.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_contents() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the extension schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "contents");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        (
            "module",
            json!({"modules": {"my_hook": {"type": "hook", "lib": "src/hook"}}}),
        ),
        (
            "modules",
            json!({"modules": {
                "my_hook": {"type": "hook", "lib": "src/hook"},
                "preload": {"type": "hook", "lib": "src/hook", "preload": "server"},
            }}),
        ),
        (
            "extension",
            json!({"extensions": {
                "my_ext": {"sql": "widget.sql", "control": "widget.control"},
            }}),
        ),
        (
            "extensions",
            json!({"extensions": {
                "my_ext": {"sql": "widget.sql", "control": "widget.control"},
                "ext2": {
                    "sql": "widget.sql",
                    "control": "widget.control",
                    "abstract": "This and that",
                }
            }}),
        ),
        ("app", json!({"apps": {"sqitch": {"bin": "sqitch"}}})),
        (
            "apps",
            json!({"apps": {
                "sqitch": {"bin": "sqitch"},
                "bog": {"bin": "bog", "lang": "perl"}
            }}),
        ),
        (
            "all three",
            json!({
                "apps": {
                    "sqitch": {"bin": "sqitch"},
                    "bog": {"bin": "bog", "lang": "perl"}
                },
                "modules": {
                    "my_hook": {"type": "hook", "lib": "src/hook"},
                },
                "extensions": {
                   "my_ext": {"sql": "widget.sql", "control": "widget.control"},
                }
            }),
        ),
        (
            "x field",
            json!({"apps": {"sqitch": {"bin": "sqitch"}},"x_hi": true}),
        ),
        (
            "X field",
            json!({"apps": {"sqitch": {"bin": "sqitch"}},"X_yo": 42}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
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
            json!({"apps": {"sqitch": {"bin": "sqitch"}}, "foo": 1}),
        ),
        (
            "bare x_",
            json!({"apps": {"sqitch": {"bin": "sqitch"}}, "x_": 1}),
        ),
        (
            "bare X_",
            json!({"apps": {"sqitch": {"bin": "sqitch"}}, "X_": 1}),
        ),
        ("short app key", json!({"apps": {"x": {"bin": "sqitch"}}})),
        (
            "short ext key",
            json!({"extensions": {
                "x": {"sql": "widget.sql", "control": "widget.control"},
            }}),
        ),
        (
            "short mod key",
            json!({"modules": {"x": {"type": "hook", "lib": "src/hook"}}}),
        ),
        // extensions
        ("empty extensions", json!({"extensions": {}})),
        ("null extensions", json!({"extensions": null, "lib": "bgw"})),
        ("empty extensions", json!({"extensions": "", "lib": "bgw"})),
        ("number extensions", json!({"extensions": 42, "lib": "bgw"})),
        ("bool extensions", json!({"extensions": true, "lib": "bgw"})),
        ("array extensions", json!({"extensions": [], "lib": "bgw"})),
        ("object extensions", json!({"extensions": {}, "lib": "bgw"})),
        // modules
        ("empty modules", json!({"modules": {}})),
        ("null modules", json!({"modules": null, "lib": "bgw"})),
        ("empty modules", json!({"modules": "", "lib": "bgw"})),
        ("number modules", json!({"modules": 42, "lib": "bgw"})),
        ("bool modules", json!({"modules": true, "lib": "bgw"})),
        ("array modules", json!({"modules": [], "lib": "bgw"})),
        ("object modules", json!({"modules": {}, "lib": "bgw"})),
        // apps
        ("empty apps", json!({"apps": {}})),
        ("null apps", json!({"apps": null, "lib": "bgw"})),
        ("empty apps", json!({"apps": "", "lib": "bgw"})),
        ("number apps", json!({"apps": 42, "lib": "bgw"})),
        ("bool apps", json!({"apps": true, "lib": "bgw"})),
        ("array apps", json!({"apps": [], "lib": "bgw"})),
        ("object apps", json!({"apps": {}, "lib": "bgw"})),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_meta_spec() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "meta-spec");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_meta_spec in [
        ("version 2.0.0 only", json!({"version": "2.0.0"})),
        ("version 2.0.1 only", json!({"version": "2.0.1"})),
        ("version 2.0.2 only", json!({"version": "2.0.2"})),
        ("version 2.0.99 only", json!({"version": "2.0.99"})),
        ("x key", json!({"version": "2.0.99", "x_y": true})),
        ("X key", json!({"version": "2.0.99", "X_x": true})),
        (
            "version plus URL",
            json!({"version": "2.0.0", "url": "https://rfcs.pgxn.org/0003-meta-spec-v2.html"}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_meta_spec.1, idx) {
            panic!("{} failed: {e}", valid_meta_spec.0);
        }
    }

    for invalid_meta_spec in [
        ("array", json!([])),
        ("string", json!("2.0.0")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("unknown field", json!({"version": "2.0.0", "foo": "hi"})),
        ("bare x_", json!({"version": "2.0.0", "x_": "hi"})),
        ("version 1.2.0", json!({"version": "1.2.0"})),
        ("version 2.2.0", json!({"version": "2.2.0"})),
        (
            "no_version",
            json!({"url": "https://rfcs.pgxn.org/0003-meta-spec-v2.html"}),
        ),
        (
            "invalid url",
            json!({"version": "2.0.1", "url": "https://pgxn.org/meta/spec.html"}),
        ),
    ] {
        if schemas.validate(&invalid_meta_spec.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_meta_spec.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_categories() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "categories");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_cats in [
        ("Analytics", json!(["Analytics"])),
        ("Auditing and Logging", json!(["Auditing and Logging"])),
        ("Change Data Capture", json!(["Change Data Capture"])),
        ("Connectors", json!(["Connectors"])),
        (
            "Data and Transformations",
            json!(["Data and Transformations"]),
        ),
        ("Debugging", json!(["Debugging"])),
        (
            "Index and Table Optimizations",
            json!(["Index and Table Optimizations"]),
        ),
        ("Machine Learning", json!(["Machine Learning"])),
        ("Metrics", json!(["Metrics"])),
        ("Orchestration", json!(["Orchestration"])),
        ("Procedural Languages", json!(["Procedural Languages"])),
        ("Query Optimizations", json!(["Query Optimizations"])),
        ("Search", json!(["Search"])),
        ("Security", json!(["Security"])),
        ("Tooling and Admin", json!(["Tooling and Admin"])),
        ("two categories", json!(["Analytics", "Debugging"])),
        (
            "three categories",
            json!(["Analytics", "Debugging", "Metrics"]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_cats.1, idx) {
            panic!("{} failed: {e}", valid_cats.0);
        }
    }

    for invalid_cats in [
        ("empty array", json!([])),
        ("string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("true category", json!([true])),
        ("false category", json!([false])),
        ("null category", json!([null])),
        ("object category", json!([{}])),
        ("empty category", json!([""])),
        ("object category", json!({})),
        ("invalid", json!(["Hackers Convention"])),
        ("dupe", json!(["Metrics", "Metrics"])),
        (
            "too many",
            json!(["Analytics", "Debugging", "Metrics", "Security"]),
        ),
    ] {
        if schemas.validate(&invalid_cats.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_cats.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_classifications() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "classifications");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        ("one tag", json!({"tags": ["xy"]})),
        ("one cat", json!({"categories": ["Metrics"]})),
        (
            "one each",
            json!({"tags": ["xy"], "categories": ["Metrics"]}),
        ),
        ("unicode tag", json!({"tags": ["😀🍒📸"]})),
        ("space tag", json!({"tags": ["hi there"]})),
        (
            "two categories",
            json!({"categories": ["Analytics", "Debugging"]}),
        ),
        (
            "three categories",
            json!({"categories": ["Analytics", "Debugging", "Metrics"]}),
        ),
        ("x field", json!({"tags": ["xy"], "x_hi": true})),
        ("X field", json!({"tags": ["xy"], "X_bar": 42})),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid_cats in [
        ("empty array", json!([])),
        ("string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("true item", json!([true])),
        ("false item", json!([false])),
        ("null item", json!([null])),
        ("object item", json!([{}])),
        ("empty item", json!([""])),
        ("object item", json!([{}])),
        ("empty tags", json!({"tags": []})),
        ("empty tag", json!({"tags": [""]})),
        ("dupe tag", json!({"tags": ["x", "x"]})),
        ("empty categories", json!({"categories": []})),
        ("invalid category", json!({"categories": ["Bogus"]})),
        (
            "dupe category",
            json!({"categories": ["Metrics", "Metrics"]}),
        ),
        (
            "too many",
            json!(["Analytics", "Debugging", "Metrics", "Security"]),
        ),
        ("unknown field", json!({"tags": ["xy"], "foo": 1})),
        ("bare x_", json!({"tags": ["xy"], "x_": 1})),
        ("bare X_", json!({"tags": ["xy"], "X_": 1})),
    ] {
        if schemas.validate(&invalid_cats.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_cats.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_ignore() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "ignore");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid ignores.
    for valid in [
        ("README.txt", json!(["README.txt"])),
        ("/.git", json!(["/.git"])),
        ("/src/pair.c", json!(["/src/pair.c"])),
        ("/src/private.*", json!(["/src/private.*"])),
        ("*.html", json!(["*.html"])),
        ("*.?tml", json!(["*.?tml"])),
        ("foo.?tml", json!(["foo.?tml"])),
        ("[xX]_*.*", json!(["[xX]_*.*"])),
        ("[a-z]*.txt", json!(["[a-z]*.txt"])),
        (
            "this\\\\and\\\\that.txt",
            json!(["this\\\\and\\\\that.txt"]),
        ),
        (
            "multiple files",
            json!(["ignore_me.txt", "*.tmp", ".git*",]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    // Test invalid ignores.
    for invalid in [
        ("empty array", json!([])),
        ("string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("true item", json!([true])),
        ("false item", json!([false])),
        ("null item", json!([null])),
        ("object item", json!([{}])),
        ("empty item", json!([""])),
        ("object item", json!([{}])),
        ("backslashes", json!("this\\and\\that.txt")),
        ("windows", json!("C:\\foo")),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_phase() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "phase");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_prereq_phase in [
        (
            "requires",
            json!({"requires": {"pkg:pgxn/citext": "2.0.0"}}),
        ),
        (
            "recommends",
            json!({"recommends": {"pkg:pgxn/citext": "2.0.0"}}),
        ),
        (
            "suggests",
            json!({"suggests": {"pkg:pgxn/citext": "2.0.0"}}),
        ),
        (
            "conflicts",
            json!({"conflicts": {"pkg:pgxn/citext": "2.0.0"}}),
        ),
        (
            "two phases",
            json!({
                "requires": {"pkg:pgxn/citext": "1.0.0"},
                "recommends": {"pkg:pgxn/citext": "2.0.0"},
            }),
        ),
        (
            "three phases",
            json!({
                "requires": {"pkg:pgxn/citext": "1.0.0"},
                "recommends": {"pkg:pgxn/citext": "2.0.0"},
                "suggests": {"pkg:pgxn/citext": "3.0.0"},
            }),
        ),
        (
            "four phases",
            json!({
                "requires": {"pkg:pgxn/citext": "1.0.0"},
                "recommends": {"pkg:pgxn/citext": "2.0.0"},
                "suggests": {"pkg:pgxn/citext": "3.0.0"},
                "conflicts": { "pkg:pypi/alligator": 0}
            }),
        ),
        ("bare zero", json!({"requires": {"pkg:pgxn/citext": 0}})),
        ("string zero", json!({"requires": {"pkg:pgxn/citext": "0"}})),
        (
            "range op",
            json!({"requires": {"pkg:pgxn/citext": "==2.0.0"}}),
        ),
        (
            "range",
            json!({"requires": {"pkg:pgxn/citext": ">= 1.2.0, != 1.5.0, < 2.0.0"}}),
        ),
        (
            "x_ field",
            json!({"requires": {"pkg:pgxn/citext": "2.0.0"}, "x_y": 1}),
        ),
        (
            "X_ field",
            json!({"requires": {"pkg:pgxn/citext": "2.0.0"}, "X_y": 1}),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid_prereq_phase.1, idx) {
            panic!("{} failed: {e}", valid_prereq_phase.0);
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
fn test_v2_packages() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "packages");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        (
            "run",
            json!({"run": {"requires": {"pkg:pgxn/citext": "2.0.0"}}}),
        ),
        (
            "build",
            json!({"build": {"requires": {"pkg:pgxn/citext": "2.0.0"}}}),
        ),
        (
            "test",
            json!({"test": {"requires": {"pkg:pgxn/citext": "2.0.0"}}}),
        ),
        (
            "configure",
            json!({"configure": {"requires": {"pkg:pgxn/citext": "2.0.0"}}}),
        ),
        (
            "develop",
            json!({"develop": {"requires": {"pkg:pgxn/citext": "2.0.0"}}}),
        ),
        (
            "two phases",
            json!({
                "build": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "test": {"requires": {"pkg:pgxn/citext": "2.0.0"}}
            }),
        ),
        (
            "three phases",
            json!({
                "configure": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "build": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "test": {"requires": {"pkg:pgxn/citext": "2.0.0"}}
            }),
        ),
        (
            "four phases",
            json!({
                "configure": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "build": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "test": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "run": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
            }),
        ),
        (
            "all phases",
            json!({
                "configure": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "build": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "test": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "run": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "develop": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
            }),
        ),
        (
            "run plus custom field",
            json!({
                "run": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "x_Y": 0,
            }),
        ),
        (
            "all phases plus custom",
            json!({
                "configure": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "build": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "test": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "run": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "develop": {"requires": {"pkg:pgxn/citext": "2.0.0"}},
                "x_Y": 0,
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
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
        // run
        ("run array", json!({"run": ["2.0.0"]})),
        ("run object", json!({"run": {}})),
        ("run string", json!({"run": "2.0.0"})),
        ("run bool", json!({"run": true})),
        ("run number", json!({"run": 42})),
        ("run null", json!({"run": null})),
        // develop
        ("develop array", json!({"develop": ["2.0.0"]})),
        ("develop object", json!({"develop": {}})),
        ("develop string", json!({"develop": "2.0.0"})),
        ("develop bool", json!({"develop": true})),
        ("develop number", json!({"develop": 42})),
        ("develop null", json!({"develop": null})),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_postgres() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "postgres");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        ("version", json!({"version": "14.0"})),
        ("version 0", json!({"version": 0})),
        ("range", json!({"version": ">=14.0, <18.1"})),
        ("with xml", json!({"version": "14.0", "with": ["xml"]})),
        ("custom x_", json!({"version": "14.0", "x_y": 1})),
        ("custom X_", json!({"version": "14.0", "X_z": true})),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("only x_", json!({"x_y": 0})),
        ("only X_", json!({"X_y": 0})),
        ("bare x_", json!({"version": 0, "x_": 0})),
        ("bare X_", json!({"version": 0, "x_": 0})),
        ("unknown", json!({"version": 0, "foo": 0})),
        // version
        ("version array", json!({"version": ["2.0.0"]})),
        ("version object", json!({"version": {}})),
        ("version empty string", json!({"version": ""})),
        ("version bool", json!({"version": true})),
        ("version number", json!({"version": 42})),
        ("version null", json!({"version": null})),
        ("version invalid", json!({"version": "xyz"})),
        // with
        ("with empty array", json!({"with": []})),
        ("with object", json!({"with": {}})),
        ("with string", json!({"with": "2.0.0"})),
        ("with bool", json!({"with": true})),
        ("with number", json!({"with": 42})),
        ("with null", json!({"with": null})),
        ("with empty string", json!({"with": [""]})),
        ("with too short", json!({"with": ["x"]})),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_pipeline() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the semver schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "pipeline");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Test valid pipelines.
    for valid in [
        json!("pgxs"),
        json!("meson"),
        json!("pgrx"),
        json!("autoconf"),
        json!("cmake"),
        json!("npm"),
        json!("cpanm"),
        json!("go"),
        json!("cargo"),
    ] {
        if let Err(e) = schemas.validate(&valid, idx) {
            panic!("{} failed: {e}", valid);
        }
    }

    // Test invalid pipelines.
    for invalid in [
        json!("vroom"),
        json!("🎃🎃"),
        json!("pgx"),
        json!(""),
        json!(true),
        json!(false),
        json!(null),
        json!([]),
        json!({}),
    ] {
        if schemas.validate(&invalid, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid)
        }
    }
    Ok(())
}

#[test]
fn test_v2_dependencies() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "dependencies");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        ("postgres", json!({"postgres": {"version": "14"}})),
        (
            "pg with",
            json!({"postgres": {"version": "14", "with": ["xml"]}}),
        ),
        ("any", json!({"platforms": ["any"]})),
        ("linux", json!({"platforms": ["linux"]})),
        ("2platform", json!({"platforms": ["linux", "darwin"]})),
        ("pgxs", json!({"pipeline": "pgxs"})),
        ("pgrx", json!({"pipeline": "pgrx"})),
        (
            "configure",
            json!({"packages": {
                "configure": { "requires": { "pkg:generic/cmake": 0} }
            }}),
        ),
        (
            "test",
            json!({"packages": {
                "test": { "requires": { "pkg:pgxn/pgtap": "1.0.0" } }
            }}),
        ),
        (
            "packages",
            json!({"packages": {
                "configure": { "requires": { "pkg:generic/cmake": 0 } },
                "build": { "recommends": { "pkg:generic/jq": 0 } },
                "test": { "requires": { "pkg:pgxn/pgtap": "1.0.0" } },
                "run": { "suggests": { "pkg:postgres/hstore": 0 } },
                "develop": { "suggests": { "pkg:generic/python": 0 } },
            }}),
        ),
        (
            "variation",
            json!({"variations": [
                {
                    "where": { "platforms": ["darwin", "bsd"] },
                    "dependencies": {"postgres": {"version": "14"}},
                }
            ]}),
        ),
        (
            "variations",
            json!({"variations": [
                {
                    "where": { "platforms": ["darwin", "bsd"] },
                    "dependencies": {"postgres": {"version": "14"}},
                },
                {
                    "where": { "postgres": { "version": ">= 16.0" } },
                    "dependencies": {
                    "postgres": { "version": ">= 16.0", "with": ["zstd"] }
                    }
                },
            ]}),
        ),
        ("custom x_", json!({"pipeline": "pgxs", "x_y": 1})),
        ("custom X_", json!({"pipeline": "pgxs", "X_z": true})),
        (
            "everything",
            json!({
                "postgres": {"version": "14", "with": ["xml"]},
                "platforms": ["linux", "darwin"],
                "pipeline": "pgrx",
                "packages": {
                    "configure": { "requires": { "pkg:generic/cmake": 0 } },
                    "build": { "recommends": { "pkg:generic/jq": 0 } },
                    "test": { "requires": { "pkg:pgxn/pgtap": "1.0.0" } },
                    "run": { "suggests": { "pkg:postgres/hstore": 0 } },
                    "develop": { "suggests": { "pkg:generic/python": 0 } },
                },
                "variations": [
                    {
                        "where": { "platforms": ["darwin", "bsd"] },
                        "dependencies": {"postgres": {"version": "14"}},
                    },
                ]
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("only x_", json!({"x_y": 0})),
        ("only X_", json!({"X_y": 0})),
        ("bare x_", json!({"pipeline": "pgxs", "x_": 0})),
        ("bare X_", json!({"pipeline": "pgxs", "x_": 0})),
        ("unknown", json!({"pipeline": "pgxs", "foo": 0})),
        // postgres
        ("postgres array", json!({"postgres": ["2.0.0"]})),
        ("postgres empty", json!({"postgres": {}})),
        ("postgres string", json!({"postgres": ""})),
        ("postgres bool", json!({"postgres": true})),
        ("postgres number", json!({"postgres": 42})),
        ("postgres null", json!({"postgres": null})),
        (
            "postgres version array",
            json!({"postgres": {"version": ["2.0.0"]}}),
        ),
        (
            "postgres version empty",
            json!({"postgres": {"version": [""]}}),
        ),
        (
            "postgres version bool",
            json!({"postgres": {"version": [true]}}),
        ),
        (
            "postgres version number",
            json!({"postgres": {"version": [42]}}),
        ),
        (
            "postgres version null",
            json!({"postgres": {"version": [null]}}),
        ),
        (
            "postgres version invalid",
            json!({"postgres": {"version": "x.y.z"}}),
        ),
        ("postgres with empty", json!({"postgres": {"with": []}})),
        ("postgres with null", json!({"postgres": {"with": null}})),
        ("postgres with bool", json!({"postgres": {"with": true}})),
        ("postgres with number", json!({"postgres": {"with": 42}})),
        ("postgres with object", json!({"postgres": {"with": {}}})),
        (
            "postgres with empty string item",
            json!({"postgres": {"with": [""]}}),
        ),
        (
            "postgres with short string item",
            json!({"postgres": {"with": ["x"]}}),
        ),
        (
            "postgres with null item",
            json!({"postgres": {"with": [null]}}),
        ),
        (
            "postgres with bool item",
            json!({"postgres": {"with": [false]}}),
        ),
        (
            "postgres with number item",
            json!({"postgres": {"with": [42]}}),
        ),
        (
            "postgres with array item",
            json!({"postgres": {"with": [["xml"]]}}),
        ),
        (
            "postgres with object item",
            json!({"postgres": {"with": {}}}),
        ),
        // platforms
        ("platforms empty", json!({"platforms": []})),
        ("platforms object", json!({"platforms": {}})),
        ("platforms string", json!({"platforms": ""})),
        ("platforms bool", json!({"platforms": true})),
        ("platforms number", json!({"platforms": 42})),
        ("platforms null", json!({"platforms": null})),
        ("platforms empty string", json!({"platforms": [""]})),
        ("platforms short string", json!({"platforms": ["x"]})),
        ("platforms item array", json!({"platforms": [[]]})),
        ("platforms item object", json!({"platforms": [{}]})),
        ("platforms item empty string", json!({"platforms": [""]})),
        ("platforms item bool", json!({"platforms": [true]})),
        ("platforms item number", json!({"platforms": [42]})),
        ("platforms item null", json!({"platforms": [null]})),
        // pipeline
        ("pipeline empty", json!({"pipeline": ""})),
        ("pipeline invalid", json!({"pipeline": "nope"})),
        ("pipeline object", json!({"pipeline": {}})),
        ("pipeline bool", json!({"pipeline": true})),
        ("pipeline number", json!({"pipeline": 42})),
        ("pipeline null", json!({"pipeline": null})),
        // packages
        ("packages array", json!({"packages": []})),
        ("packages empty", json!({"packages": {}})),
        ("packages string", json!({"packages": ""})),
        ("packages bool", json!({"packages": true})),
        ("packages number", json!({"packages": 42})),
        ("packages null", json!({"packages": null})),
        // configure
        (
            "packages configure array",
            json!({"packages": {"configure": []}}),
        ),
        ("packages build empty", json!({"packages": {"build": {}}})),
        ("packages test string", json!({"packages": {"test": "hi"}})),
        ("packages run bool", json!({"packages": {"run": true}})),
        ("packages run null", json!({"packages": {"run": null}})),
        (
            "packages develop number",
            json!({"packages": {"develop": 42}}),
        ),
        // variations
        ("variations empty", json!({"variations": []})),
        ("variations object", json!({"variations": {}})),
        ("variations string", json!({"variations": ""})),
        ("variations bool", json!({"variations": true})),
        ("variations number", json!({"variations": 42})),
        ("variations null", json!({"variations": null})),
        (
            "nested where variations",
            json!({"variations": [
                {
                    "where": {
                        "platforms": ["darwin", "bsd"],
                        "variations": {"pipeline": "pgxs"},
                    },
                    "dependencies": {
                        "postgres": {"version": "14"},
                    },
                }
            ]}),
        ),
        (
            "nested dependencies variations",
            json!({"variations": [
                {
                    "where": { "platforms": ["darwin", "bsd"] },
                    "dependencies": {
                        "postgres": {"version": "14"},
                        "variations": {"pipeline": "pgxs"},
                    },
                }
            ]}),
        ),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_variations() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "variations");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        (
            "one",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
            }]),
        ),
        (
            "two",
            json!([
                {
                    "where": { "platforms": ["darwin", "bsd"] },
                    "dependencies": {"postgres": {"version": "14"}},
                },
                {
                    "where": { "postgres": { "version": ">= 16.0" } },
                    "dependencies": {
                    "postgres": { "version": ">= 16.0", "with": ["zstd"] }
                    }
                },
            ]),
        ),
        (
            "with x_",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
                "x_y": true,
            }]),
        ),
        (
            "with X_",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
                "X_y": 42,
            }]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("empty", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("only x_", json!({"x_y": 0})),
        ("only X_", json!({"X_y": 0})),
        (
            "no dependencies",
            json!({"where": { "platforms": ["darwin", "bsd"] }}),
        ),
        (
            "no where",
            json!({"dependencies": { "platforms": ["darwin", "bsd"] }}),
        ),
        (
            "bare x_",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
                "x_": true,
            }]),
        ),
        (
            "bare X_",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
                "X_": 42,
            }]),
        ),
        (
            "unknown x_",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {"postgres": {"version": "14"}},
                "foo": true,
            }]),
        ),
        (
            "nested where",
            json!([{
                "where": {
                    "platforms": ["darwin", "bsd"],
                    "variations": {
                        "where": { "platforms": ["darwin", "bsd"] },
                        "dependencies": {"postgres": {"version": "14"}},
                    },
                 },
                "dependencies": {"postgres": {"version": "14"}},
            }]),
        ),
        (
            "nested dependencies",
            json!([{
                "where": { "platforms": ["darwin", "bsd"] },
                "dependencies": {
                    "postgres": {"version": "14"},
                    "variations": {
                        "where": { "platforms": ["darwin", "bsd"] },
                        "dependencies": {"postgres": {"version": "14"}},
                    },
                },
            }]),
        ),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_badges() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "badges");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        ("short", json!([{"src": "x:y", "alt": "food"}])),
        (
            "long",
            json!([{
                "src": "https://github.com/theory/kv-pair/workflows/CI/badge.svg",
                "alt": "CI/CD Test Status",
                "url": "https://github.com/theory/pgtap/actions/workflows/ci.yml"
            }]),
        ),
        (
            "multi",
            json!([
                {"src": "x:y", "alt": "food"},
                {"src": "a:b", "alt": "tests"},
                {"src": "mailto:x@example.com", "alt": "Contact Me!"},
            ]),
        ),
        (
            "custom x_",
            json!([{"src": "x:y", "alt": "food", "x_y": 1}]),
        ),
        (
            "custom X_",
            json!([{"src": "x:y", "alt": "food", "X_z": true}]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("empty array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("empty item", json!([{}])),
        ("array item", json!([[]])),
        ("string item", json!([""])),
        ("null item", json!([null])),
        ("number item", json!([42])),
        ("bool item", json!([true])),
        ("only x_", json!([{"x_y": 0}])),
        ("only X_", json!([{"X_y": 0}])),
        ("bare x_", json!([{"src": "x:y", "alt": "food", "x_": 0}])),
        ("bare X_", json!([{"src": "x:y", "alt": "food", "x_": 0}])),
        ("unknown", json!([{"src": "x:y", "alt": "food", "foo": 0}])),
        // src
        ("src array", json!([{"alt": "abcd", "src": []}])),
        ("src object", json!([{"alt": "abcd", "src": {}}])),
        ("src empty", json!([{"alt": "abcd", "src": ""}])),
        ("src bool", json!([{"alt": "abcd", "src": true}])),
        ("src number", json!([{"alt": "abcd", "src": 42}])),
        ("src null", json!([{"alt": "abcd", "src": null}])),
        ("src invalid", json!([{"alt": "abcd", "src": "not a uri"}])),
        // alt
        ("alt array", json!([{"src": "x:y", "alt": []}])),
        ("alt object", json!([{"src": "x:y", "alt": {}}])),
        ("alt empty", json!([{"src": "x:y", "alt": ""}])),
        ("alt bool", json!([{"src": "x:y", "alt": true}])),
        ("alt number", json!([{"src": "x:y", "alt": 42}])),
        ("alt null", json!([{"src": "x:y", "alt": null}])),
        ("alt too short", json!([{"src": "x:y", "alt": "xyz"}])),
        // url
        (
            "url array",
            json!([{"src": "x:y", "alt": "abcd", "url": []}]),
        ),
        (
            "url object",
            json!([{"src": "x:y", "alt": "abcd", "url": {}}]),
        ),
        (
            "url empty",
            json!([{"src": "x:y", "alt": "abcd", "url": ""}]),
        ),
        (
            "url bool",
            json!([{"src": "x:y", "alt": "abcd", "url": true}]),
        ),
        (
            "url number",
            json!([{"src": "x:y", "alt": "abcd", "url": 42}]),
        ),
        (
            "url null",
            json!([{"src": "x:y", "alt": "abcd", "url": null}]),
        ),
        (
            "url invalid",
            json!([{"src": "x:y", "alt": "abcd", "url": "not a url"}]),
        ),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_resources() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "resources");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        ("homepage", json!({"homepage": "https://example.com"})),
        ("issues", json!({"issues": "https://github.com/issues"})),
        ("repo", json!({"repository": "https://github.com/repo"})),
        ("docs", json!({"repository": "https://example.com"})),
        ("support", json!({"repository": "https://example.com"})),
        ("badges", json!({"badges": [{"src": "x:y", "alt": "abcd"}]})),
        ("custom x_", json!({"docs": "x:y", "x_y": 1})),
        ("custom X_", json!({"docs": "x:y", "X_z": true})),
        (
            "everything",
            json!({
                "homepage": "https://pgtap.org",
                "issues": "https://github.com/theory/pgtap/issues",
                "repository": "https://github.com/theory/pgtap",
                "docs": "https://pgtap.org/documentation.html",
                "support": "https://github.com/theory/pgtap",
                "badges": [
                    {
                        "src": "https://img.shields.io/badge/License-PostgreSQL-blue.svg",
                        "alt": "PostgreSQL License",
                        "url": "https://www.postgresql.org/about/licence/"
                    },
                    {
                        "src": "https://github.com/theory/pgtap/actions/workflows/test.yml/badge.svg",
                        "alt": "Test Status",
                        "url": "https://github.com/theory/pgtap/actions/workflows/ci.yml"
                    },
                ]
            }),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("empty object", json!({})),
        ("only x_", json!({"x_y": 0})),
        ("only X_", json!({"X_y": 0})),
        ("bare x_", json!({"docs": "x:y", "x_": 0})),
        ("bare X_", json!({"docs": "x:y", "x_": 0})),
        ("unknown", json!({"docs": "x:y", "foo": 0})),
        // homepage
        ("homepage array", json!([{"homepage": []}])),
        ("homepage object", json!([{"homepage": {}}])),
        ("homepage empty", json!([{"homepage": ""}])),
        ("homepage bool", json!([{"homepage": true}])),
        ("homepage number", json!([{"homepage": 42}])),
        ("homepage null", json!([{"homepage": null}])),
        ("homepage empty", json!([{"homepage": ""}])),
        ("homepage invalid", json!([{"homepage": "not a uri"}])),
        // issues
        ("issues array", json!([{"issues": []}])),
        ("issues object", json!([{"issues": {}}])),
        ("issues empty", json!([{"issues": ""}])),
        ("issues bool", json!([{"issues": true}])),
        ("issues number", json!([{"issues": 42}])),
        ("issues null", json!([{"issues": null}])),
        ("issues empty", json!([{"issues": ""}])),
        ("issues invalid", json!([{"issues": "not a uri"}])),
        // repository
        ("repository array", json!([{"repository": []}])),
        ("repository object", json!([{"repository": {}}])),
        ("repository empty", json!([{"repository": ""}])),
        ("repository bool", json!([{"repository": true}])),
        ("repository number", json!([{"repository": 42}])),
        ("repository null", json!([{"repository": null}])),
        ("repository empty", json!([{"repository": ""}])),
        ("repository invalid", json!([{"repository": "not a uri"}])),
        // docs
        ("docs array", json!([{"docs": []}])),
        ("docs object", json!([{"docs": {}}])),
        ("docs empty", json!([{"docs": ""}])),
        ("docs bool", json!([{"docs": true}])),
        ("docs number", json!([{"docs": 42}])),
        ("docs null", json!([{"docs": null}])),
        ("docs empty", json!([{"docs": ""}])),
        ("docs invalid", json!([{"docs": "not a uri"}])),
        // support
        ("support array", json!([{"support": []}])),
        ("support object", json!([{"support": {}}])),
        ("support empty", json!([{"support": ""}])),
        ("support bool", json!([{"support": true}])),
        ("support number", json!([{"support": 42}])),
        ("support null", json!([{"support": null}])),
        ("support empty", json!([{"support": ""}])),
        ("support invalid", json!([{"support": "not a uri"}])),
        // badges
        ("badges empty", json!([{"badges": []}])),
        ("badges object", json!([{"badges": {}}])),
        ("badges empty", json!([{"badges": ""}])),
        ("badges bool", json!([{"badges": true}])),
        ("badges number", json!([{"badges": 42}])),
        ("badges null", json!([{"badges": null}])),
        ("badges empty", json!([{"badges": ""}])),
        ("badges src only", json!([{"badges": {"src": "x:y"}}])),
        ("badges alt only", json!([{"badges": {"alt": "abcd"}}])),
        (
            "badges src invalid",
            json!([{"badges": {"alt": "abcd", "src": "not a uri"}}]),
        ),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

#[test]
fn test_v2_artifacts() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the maintainer schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "artifacts");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid in [
        (
            "basic",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
            }]),
        ),
        (
            "two",
            json!([
                {
                    "url": "x:y",
                    "type": "bin",
                    "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                },
                {
                    "url": "a:b",
                    "type": "zip",
                    "sha512": "22E06F682A7FEC79F814F06B5DCEA0B06133890775DDC624DE744CD5D4E8D5FE29863BA5E77C6D3690B610DBCDF7D79A973561FDFBD8454508998446AF8F2C58",
                },
            ]),
        ),
        (
            "all fields",
            json!([
                {
                    "url": "x:y",
                    "type": "bin",
                    "platform": "linux",
                    "sha256": "0B68EE2CE5B2C0641C6C429ED2CE17E2ED76DDD58BF9A16E698C5069D60AA34E",
                },
                {
                    "url": "a:b",
                    "type": "zip",
                    "platform": "darwin",
                    "sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c58",
                },
            ]),
        ),
        (
            "custom x_",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "x_y": 1,
            }]),
        ),
        (
            "custom X_",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "X_y": true,
            }]),
        ),
    ] {
        if let Err(e) = schemas.validate(&valid.1, idx) {
            panic!("{} failed: {e}", valid.0);
        }
    }

    for invalid in [
        ("empty array", json!([])),
        ("string", json!("web")),
        ("empty string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("only x_", json!({"x_y": 0})),
        ("only X_", json!({"X_y": 0})),
        (
            "bare x_",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "x_": 1,
            }]),
        ),
        (
            "bare X_",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "X_": 1,
            }]),
        ),
        (
            "unknown",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "foo": 1,
            }]),
        ),
        // url
        (
            "url array",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": [],
            }]),
        ),
        (
            "url object",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": {},
            }]),
        ),
        (
            "url empty",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": "",
            }]),
        ),
        (
            "url bool",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": false,
            }]),
        ),
        (
            "url number",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": 42,
            }]),
        ),
        (
            "url null",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": null,
            }]),
        ),
        (
            "url invalid",
            json!([{
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "url": "not a uri",
            }]),
        ),
        // type
        (
            "type array",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": [],
            }]),
        ),
        (
            "type object",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": {},
            }]),
        ),
        (
            "type empty",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": "",
            }]),
        ),
        (
            "type bool",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": false,
            }]),
        ),
        (
            "type number",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": 42,
            }]),
        ),
        (
            "type null",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": null,
            }]),
        ),
        (
            "type too short",
            json!([{
                "url": "x:y",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "type": "x",
            }]),
        ),
        // sha256
        (
            "sha256 array",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": [],
            }]),
        ),
        (
            "sha256 object",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": {},
            }]),
        ),
        (
            "sha256 empty",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "",
            }]),
        ),
        (
            "sha256 bool",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": false,
            }]),
        ),
        (
            "sha256 number",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": 42,
            }]),
        ),
        (
            "sha256 null",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": null,
            }]),
        ),
        (
            "sha256 not hex",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34x",
            }]),
        ),
        (
            "sha256 too long",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34ee",
            }]),
        ),
        (
            "sha256 too short",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34",
            }]),
        ),
        // sha512
        (
            "sha512 array",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": [],
            }]),
        ),
        (
            "sha512 object",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": {},
            }]),
        ),
        (
            "sha512 empty",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": "",
            }]),
        ),
        (
            "sha512 bool",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": false,
            }]),
        ),
        (
            "sha512 number",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": 42,
            }]),
        ),
        (
            "sha512 null",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": null,
            }]),
        ),
        (
            "sha512 not hex",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c5x",
            }]),
        ),
        (
            "sha512 too long",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c58e",
            }]),
        ),
        (
            "sha512 too short",
            json!([{
                "url": "x:y",
                "type": "bin",
                "sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c5",
            }]),
        ),
    ] {
        if schemas.validate(&invalid.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid.0)
        }
    }

    Ok(())
}

fn valid_v2_distribution() -> Value {
    json!({
      "name": "pgTAP",
      "abstract": "Unit testing for PostgreSQL",
      "description": "pgTAP is a suite of database functions that make it easy to write TAP-emitting unit tests in psql scripts or xUnit-style test functions.",
      "version": "0.26.0",
      "maintainers": [
        { "name": "Josh Berkus", "email": "jberkus@pgxn.org" },
        { "name": "David E. Wheeler", "url": "https://pgxn.org/user/theory" }
      ],
      "license": "MIT OR PostgreSQL",
      "dependencies": {
        "postgres": { "version": "8.4" },
        "packages": {
          "run": {
            "requires": {
              "pkg:postgres/plpgsql": 0
            }
          }
        }
      },
      "contents": {
        "extensions": {
          "pgtap": {
            "abstract": "Unit testing for PostgreSQL",
            "sql": "pgtap.sql",
            "control": "pgtap.control"
          }
        }
      },
      "resources": {
        "homepage": "https://pgtap.org",
        "issues": "https://github.com/theory/pgtap/issues",
        "repository": "https://github.com/theory/pgtap",
        "docs": "https://pgtap.org/documentation.html",
        "support": "https://github.com/theory/pgtap",
        "badges": [
          {
            "src": "https://img.shields.io/badge/License-PostgreSQL-blue.svg",
            "alt": "PostgreSQL License",
            "url": "https://www.postgresql.org/about/licence/"
          },
          {
            "src": "https://github.com/theory/pgtap/actions/workflows/test.yml/badge.svg",
            "alt": "Test Status",
            "url": "https://github.com/theory/pgtap/actions/workflows/ci.yml"
          }
        ]
      },
      "producer": "David E. Wheeler",
      "meta-spec": {
        "version": "2.0.0",
        "url": "https://rfcs.pgxn.org/0003-meta-spec-v2.html"
      },
      "classifications": {
        "tags": [
          "testing",
          "unit testing",
          "tap",
          "tddd",
          "test driven database development"
        ],
        "categories": [ "Tooling and Admin" ]
      }
    })
}

#[test]
fn test_v2_distribution() -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the distribution schema.
    let mut compiler = new_compiler("schema/v2")?;
    let mut schemas = Schemas::new();
    let id = id_for(SCHEMA_VERSION, "distribution");
    let idx = compiler.compile(&id, &mut schemas)?;

    // Make sure the valid distribution is in fact valid.
    let meta = valid_v2_distribution();
    if let Err(e) = schemas.validate(&meta, idx) {
        panic!("valid_distribution meta failed: {e}");
    }

    type Obj = Map<String, Value>;
    type Callback = fn(&mut Obj);

    static VALID_TEST_CASES: &[(&str, Callback)] = &[
        ("no change", |_: &mut Obj| {}),
        ("custom key x_y", |m: &mut Obj| {
            m.insert("x_y".to_string(), json!("hello"));
        }),
        ("custom key X_y", |m: &mut Obj| {
            m.insert("X_y".to_string(), json!(42));
        }),
        ("license apache_2_0", |m: &mut Obj| {
            m.insert("license".to_string(), json!("Apache-2.0"));
        }),
        ("license postgresql", |m: &mut Obj| {
            m.insert("license".to_string(), json!("PostgreSQL"));
        }),
        ("license AND", |m: &mut Obj| {
            m.insert("license".to_string(), json!("MIT AND PostgreSQL"));
        }),
        ("contents extension doc", |m: &mut Obj| {
            let contents = m.get_mut("contents").unwrap().as_object_mut().unwrap();
            let ext = contents
                .get_mut("extensions")
                .unwrap()
                .as_object_mut()
                .unwrap();
            let pgtap = ext.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.insert("doc".to_string(), json!("foo/bar.txt"));
        }),
        ("contents extension no abstract", |m: &mut Obj| {
            let contents = m.get_mut("contents").unwrap().as_object_mut().unwrap();
            let ext = contents
                .get_mut("extensions")
                .unwrap()
                .as_object_mut()
                .unwrap();
            let pgtap = ext.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.remove("abstract");
        }),
        ("add modules", |m: &mut Obj| {
            let contents = m.get_mut("contents").unwrap().as_object_mut().unwrap();
            contents.insert(
                "modules".to_string(),
                json!({"my_hook": {"type": "hook", "lib": "src/hook"}}),
            );
        }),
        ("add apps", |m: &mut Obj| {
            let contents = m.get_mut("contents").unwrap().as_object_mut().unwrap();
            contents.insert(
                "apps".to_string(),
                json!({
                    "sqitch": {"bin": "sqitch"},
                    "bog": {"bin": "bog", "lang": "perl"}
                }),
            );
        }),
        ("no spec URL", |m: &mut Obj| {
            let spec = m.get_mut("meta-spec").unwrap().as_object_mut().unwrap();
            spec.remove("url");
        }),
        ("multibyte name", |m: &mut Obj| {
            m.insert("name".to_string(), json!("yoŭknow"));
        }),
        ("emoji name", |m: &mut Obj| {
            m.insert("name".to_string(), json!("📀📟🎱"));
        }),
        ("name with dash", |m: &mut Obj| {
            m.insert("name".to_string(), json!("foo-bar"));
        }),
        ("multibyte abstract", |m: &mut Obj| {
            m.insert("abstract".to_string(), json!("yoŭknow"));
        }),
        ("emoji abstract", |m: &mut Obj| {
            m.insert("abstract".to_string(), json!("📀📟🎱"));
        }),
        ("no producer", |m: &mut Obj| {
            m.remove("producer");
        }),
        ("one tag", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!({"tags": ["foo"]}));
        }),
        ("no ignore", |m: &mut Obj| {
            m.remove("ignore");
        }),
        ("no resources", |m: &mut Obj| {
            m.remove("resources");
        }),
        ("one resource", |m: &mut Obj| {
            m.insert(
                "resources".to_string(),
                json!({"docs": "https://example.com"}),
            );
        }),
        ("on maintainer", |m: &mut Obj| {
            m.insert(
                "maintainers".to_string(),
                json!([{"name": "Hi There", "url": "x:y"}]),
            );
        }),
        ("pre-release version", |m: &mut Obj| {
            m.insert("version".to_string(), json!("1.2.1-beta1"));
        }),
        ("multibyte description", |m: &mut Obj| {
            m.insert("description".to_string(), json!("yoŭknow"));
        }),
        ("emoji description", |m: &mut Obj| {
            m.insert("description".to_string(), json!("📀📟🎱"));
        }),
        ("multibyte producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!("yoŭknow"));
        }),
        ("emoji producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!("📀📟🎱"));
        }),
        ("postgres dependencies", |m: &mut Obj| {
            m.insert(
                "dependencies".to_string(),
                json!({"postgres": { "version": "12", "with": ["xml"] }}),
            );
        }),
        ("dependency variations", |m: &mut Obj| {
            m.insert(
                "dependencies".to_string(),
                json!({"variations": [{
                  "where": {"postgres": { "version": "16" }},
                  "dependencies": {"platforms": ["linux"]},
                }]}),
            );
        }),
        ("artifacts", |m: &mut Obj| {
            m.insert(
                "artifacts".to_string(),
                json!([{
                    "type": "source",
                    "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0.zip",
                    "sha256": "2b9d2416096d2930be51e5332b70bcd97846947777a93e4a3d65fe1b5fd7b004"
                }]),
            );
        }),
    ];

    for tc in VALID_TEST_CASES {
        let mut meta = valid_v2_distribution();
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
        ("no maintainers", |m: &mut Obj| {
            m.remove("maintainers");
        }),
        ("no license", |m: &mut Obj| {
            m.remove("license");
        }),
        ("no meta-spec", |m: &mut Obj| {
            m.remove("meta-spec");
        }),
        ("no contents", |m: &mut Obj| {
            m.remove("contents");
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
        ("contents no control", |m: &mut Obj| {
            let contents = m.get_mut("contents").unwrap().as_object_mut().unwrap();
            let ext = contents
                .get_mut("extensions")
                .unwrap()
                .as_object_mut()
                .unwrap();
            let pgtap = ext.get_mut("pgtap").unwrap().as_object_mut().unwrap();
            pgtap.remove("control");
        }),
        ("no postgres version", |m: &mut Obj| {
            let deps = m.get_mut("dependencies").unwrap().as_object_mut().unwrap();
            deps.insert("postgres".to_string(), json!({"with": ["xml"]}));
        }),
        ("invalid key", |m: &mut Obj| {
            m.insert("foo".to_string(), json!(1));
        }),
        ("invalid license", |m: &mut Obj| {
            m.insert("license".to_string(), json!("gobbledygook"));
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
        ("empty", |m: &mut Obj| {
            m.insert("name".to_string(), json!(""));
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
        ("empty producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!(""));
        }),
        ("null producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!(null));
        }),
        ("array producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!([]));
        }),
        ("object producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!({}));
        }),
        ("bool producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!(false));
        }),
        ("number producer", |m: &mut Obj| {
            m.insert("producer".to_string(), json!(42));
        }),
        ("null classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!(null));
        }),
        ("string classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!(""));
        }),
        ("array classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!([]));
        }),
        ("empty classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!({}));
        }),
        ("bool classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!(false));
        }),
        ("number classifications", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!(42));
        }),
        ("null tag", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!({"tags": [null]}));
        }),
        ("short tag", |m: &mut Obj| {
            m.insert("classifications".to_string(), json!({"tags": ["x"]}));
        }),
        ("ignore null file string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": null}));
        }),
        ("ignore null file empty array", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": []}));
        }),
        ("ignore null file object", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": {}}));
        }),
        ("ignore null file bool", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": true}));
        }),
        ("ignore null file number", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": 42}));
        }),
        ("ignore empty file array string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [""]}));
        }),
        ("ignore undef file array string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [null]}));
        }),
        ("ignore undef file array number", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [42]}));
        }),
        ("ignore undef file array bool", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [true]}));
        }),
        ("ignore undef file array obj", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [{}]}));
        }),
        ("ignore undef file array array", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"file": [[]]}));
        }),
        ("ignore empty directory string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": ""}));
        }),
        ("ignore null directory string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": null}));
        }),
        ("ignore null directory empty array", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": []}));
        }),
        ("ignore null directory object", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": {}}));
        }),
        ("ignore null directory bool", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": true}));
        }),
        ("ignore null directory number", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": 42}));
        }),
        ("ignore empty directory array string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [""]}));
        }),
        ("ignore undef directory array string", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [null]}));
        }),
        ("ignore undef directory array number", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [42]}));
        }),
        ("ignore undef directory array bool", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [true]}));
        }),
        ("ignore undef directory array obj", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [{}]}));
        }),
        ("ignore undef directory array array", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"directory": [[]]}));
        }),
        ("ignore bad key", |m: &mut Obj| {
            m.insert("ignore".to_string(), json!({"foo": "hi"}));
        }),
        ("null resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!(null));
        }),
        ("array resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!([]));
        }),
        ("empty resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!({}));
        }),
        ("string resources", |m: &mut Obj| {
            m.insert("resources".to_string(), json!(""));
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
        ("object artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!({}));
        }),
        ("null artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!(null));
        }),
        ("empty artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!([]));
        }),
        ("string artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!(""));
        }),
        ("bool artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!(true));
        }),
        ("number artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!(42));
        }),
        ("object artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!({}));
        }),
        ("array artifacts", |m: &mut Obj| {
            m.insert("artifacts".to_string(), json!([]));
        }),
        ("invalid artifact sha", |m: &mut Obj| {
            m.insert(
                "artifacts".to_string(),
                json!([{"url": "x:y", "type": "bin", "sha256": {}}]),
            );
        }),
    ];
    for tc in INVALID_TEST_CASES {
        let mut meta = valid_v2_distribution();
        let map = meta.as_object_mut().unwrap();
        tc.1(map);
        if schemas.validate(&meta, idx).is_ok() {
            panic!("{} unexpectedly passed!", tc.0)
        }
    }

    Ok(())
}
