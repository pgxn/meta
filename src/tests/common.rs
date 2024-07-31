use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    path::Path,
};

use crate::valid::compiler;
use boon::{Compiler, Schemas};
use serde_json::{json, Value};
use wax::Glob;

const SCHEMA_BASE: &str = "https://pgxn.org/meta/v";

// https://regex101.com/r/Ly7O1x/3/
pub const VALID_SEMVERS: &[&str] = &[
    "0.0.4",
    "1.2.3",
    "10.20.30",
    "1.1.2-prerelease+meta",
    "1.1.2+meta",
    "1.1.2+meta-valid",
    "1.0.0-alpha",
    "1.0.0-beta",
    "1.0.0-alpha.beta",
    "1.0.0-alpha.beta.1",
    "1.0.0-alpha.1",
    "1.0.0-alpha0.valid",
    "1.0.0-alpha.0valid",
    "1.0.0-alpha-a.b-c-something-long+build.1-aef.1-its-okay",
    "1.0.0-rc.1+build.1",
    "2.0.0-rc.1+build.123",
    "1.2.3-beta",
    "10.2.3-DEV-SNAPSHOT",
    "1.2.3-SNAPSHOT-123",
    "1.0.0",
    "2.0.0",
    "1.1.7",
    "2.0.0+build.1848",
    "2.0.1-alpha.1227",
    "1.0.0-alpha+beta",
    "1.2.3----RC-SNAPSHOT.12.9.1--.12+788",
    "1.2.3----R-S.12.9.1--.12+meta",
    "1.2.3----RC-SNAPSHOT.12.9.1--.12",
    "1.0.0+0.build.1-rc.10000aaa-kk-0.1",
    "1.0.0-0A.is.legal",
];

pub const INVALID_SEMVERS: &[&str] = &[
    "1",
    "1.2",
    "1.2.3-0123",
    "1.2.3-0123.0123",
    "1.1.2+.123",
    "+invalid",
    "-invalid",
    "-invalid+invalid",
    "-invalid.01",
    "alpha",
    "alpha.beta",
    "alpha.beta.1",
    "alpha.1",
    "alpha+beta",
    "alpha_beta",
    "alpha.",
    "alpha..",
    "beta",
    "1.0.0-alpha_beta",
    "-alpha.",
    "1.0.0-alpha..",
    "1.0.0-alpha..1",
    "1.0.0-alpha...1",
    "1.0.0-alpha....1",
    "1.0.0-alpha.....1",
    "1.0.0-alpha......1",
    "1.0.0-alpha.......1",
    "01.1.1",
    "1.01.1",
    "1.1.01",
    "1.2",
    "1.2.3.DEV",
    "1.2-SNAPSHOT",
    "1.2.31.2.3----RC-SNAPSHOT.12.09.1--..12+788",
    "1.2-RC-SNAPSHOT",
    "-1.0.3-gamma+b7718",
    "+just-meta",
    "9.8.7+meta+meta",
    "9.8.7-whatever+meta+meta",
    "99999999999999999999999.999999999999999999.99999999999999999----RC-SNAPSHOT.12.09.1--------------------------------..12",
];

pub fn id_for(version: u8, schema: &str) -> String {
    format!("{SCHEMA_BASE}{version}/{schema}.schema.json")
}

pub fn new_compiler<P: AsRef<Path>>(dir: P) -> Result<Compiler, Box<dyn Error>> {
    let mut compiler = compiler::spec_compiler();
    let glob = Glob::new("**/*.schema.json")?;
    for path in glob.walk(dir) {
        let path = path?.into_path();
        let schema: Value = serde_json::from_reader(File::open(&path)?)?;
        let id = &schema["$id"]
            .as_str()
            .ok_or(format!("Missing $id from {}", &path.display()))?;
        compiler.add_resource(id, schema.to_owned())?;
    }

    Ok(compiler)
}

pub fn test_term_schema(mut compiler: Compiler, version: u8) -> Result<(), Box<dyn Error>> {
    let mut schemas = Schemas::new();
    let id = id_for(version, "term");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_term in [
        ("two chars", json!("hi")),
        ("underscores", json!("hi_this_is_a_valid_term")),
        ("dashes", json!("hi-this-is-a-valid-term")),
        ("punctuation", json!("!@#$%^&*()-=+{}<>,?")),
        ("unicode", json!("😀🍒📸")),
    ] {
        if let Err(e) = schemas.validate(&valid_term.1, idx) {
            panic!("term {} failed: {e}", valid_term.0);
        }
    }

    for invalid_term in [
        ("array", json!([])),
        ("empty string", json!("")),
        ("too short", json!("x")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("space", json!("hi there")),
        ("slash", json!("hi/there")),
        ("backslash", json!("hi\\there")),
        ("null byte", json!("hi\x00there")),
    ] {
        if schemas.validate(&invalid_term.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_term.0)
        }
    }

    // Schema v1 allows a dot but v2 does not.
    let dot_term = json!("this.that");
    let res = schemas.validate(&dot_term, idx);
    if version == 1 {
        if let Err(e) = res {
            panic!("term with dot failed: {e}");
        }
    } else if res.is_ok() {
        panic!("term with dot unexpectedly passed!")
    }

    Ok(())
}

pub fn test_tags_schema(mut compiler: Compiler, version: u8) -> Result<(), Box<dyn Error>> {
    // Load the schemas and compile the tags schema.
    let mut schemas = Schemas::new();
    let id = id_for(version, "tags");
    let idx = compiler.compile(&id, &mut schemas)?;

    for valid_tags in [
        ("two chars", json!(["hi"])),
        ("underscores", json!(["hi_this_is_a_valid_tags"])),
        ("dashes", json!(["hi-this-is-a-valid-tags"])),
        ("punctuation", json!(["!@#$%^&*()-=+{}<>,.?"])),
        ("unicode", json!(["😀🍒📸"])),
        ("space", json!(["hi there"])),
        ("multiple", json!(["testing", "json", "😀🍒📸"])),
        ("max length", json!(["x".repeat(255)])),
    ] {
        if let Err(e) = schemas.validate(&valid_tags.1, idx) {
            panic!("extension {} failed: {e}", valid_tags.0);
        }
    }

    for invalid_tags in [
        ("empty array", json!([])),
        ("string", json!("")),
        ("true", json!(true)),
        ("false", json!(false)),
        ("null", json!(null)),
        ("object", json!({})),
        ("true tag", json!([true])),
        ("false tag", json!([false])),
        ("null tag", json!([null])),
        ("object tag", json!([{}])),
        ("empty tag", json!([""])),
        ("too short", json!(["x"])),
        ("object tag", json!({})),
        ("slash", json!(["hi/there"])),
        ("backslash", json!(["hi\\there"])),
        ("null byte", json!(["hi\x00there"])),
        ("too long", json!(["x".repeat(256)])),
        ("dupe", json!(["abc", "abc"])),
    ] {
        if schemas.validate(&invalid_tags.1, idx).is_ok() {
            panic!("{} unexpectedly passed!", invalid_tags.0)
        }
    }

    Ok(())
}

pub fn test_schema_version(version: u8) -> Result<(), Box<dyn Error>> {
    let mut compiler = Compiler::new();
    compiler.enable_format_assertions();
    let mut loaded: HashMap<String, Vec<Value>> = HashMap::new();

    let paths = fs::read_dir(format!("./schema/v{version}"))?;
    for path in paths {
        let path = path?.path();
        let bn = path.file_name().unwrap().to_str().unwrap();
        if bn.ends_with(".schema.json") {
            let schema: Value = serde_json::from_reader(File::open(path.clone())?)?;
            if let Value::String(s) = &schema["$id"] {
                // Make sure that the ID is correct.
                assert_eq!(format!("https://pgxn.org/meta/v{version}/{bn}"), *s);

                // Add the schema to the compiler.
                compiler.add_resource(s, schema.to_owned())?;

                // Grab the examples, if any, to test later.
                if let Value::Array(a) = &schema["examples"] {
                    loaded.insert(s.clone(), a.to_owned());
                } else {
                    loaded.insert(s.clone(), Vec::new());
                }
            } else {
                panic!("Unable to find ID in {}", path.display());
            }
        } else {
            println!("Skipping {}", path.display());
        }
    }

    // Make sure we found schemas.
    assert!(!loaded.is_empty(), "No schemas loaded!");

    // Make sure each schema we loaded is valid.
    let mut schemas = Schemas::new();
    for (id, examples) in loaded {
        let index = compiler.compile(id.as_str(), &mut schemas)?;
        println!("{} ok", id);

        // Test the schema's examples.
        for (i, example) in examples.iter().enumerate() {
            if let Err(e) = schemas.validate(example, index) {
                panic!("Example {i} failed: {e}");
            }
            // println!("  Example {i} ok");
        }
    }

    Ok(())
}
