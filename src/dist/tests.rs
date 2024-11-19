use super::*;
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    path::PathBuf,
};
use wax::Glob;

#[test]
fn test_corpus() -> Result<(), Error> {
    for v_dir in ["v1", "v2"] {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", v_dir]
            .iter()
            .collect();
        let glob = Glob::new("*.json")?;

        for path in glob.walk(dir) {
            let path = path?.into_path();
            let contents: Value = serde_json::from_reader(File::open(&path)?)?;

            // Test load path.
            if let Err(e) = Distribution::load(&path) {
                panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap());
            }

            // Test try_from str.
            let str: String = fs::read_to_string(&path)?;
            match Distribution::try_from(&str) {
                Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                Ok(dist) => {
                    if v_dir == "v2" {
                        assert_eq!(contents.get("license").unwrap(), dist.license());
                        // Make sure round-trip produces the same JSON.
                        let output: Result<String, Error> = dist.try_into();
                        match output {
                            Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                            Ok(val) => {
                                let val: Value = serde_json::from_str(&val)?;
                                assert_json_diff::assert_json_eq!(&contents, &val);
                            }
                        };
                    }
                }
            }

            // Test try_from value.
            match Distribution::try_from(contents.clone()) {
                Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                Ok(dist) => {
                    if v_dir == "v2" {
                        // Make sure value round-trip produces the same JSON.
                        let output: Result<Value, Error> = dist.try_into();
                        match output {
                            Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                            Ok(val) => {
                                assert_json_diff::assert_json_eq!(&contents, &val);
                            }
                        };
                    }
                }
            }

            println!("Example {v_dir}/{:?} ok", path.file_name().unwrap());
        }
    }
    Ok(())
}

#[test]
fn test_bad_corpus() -> Result<(), Error> {
    let file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "invalid.json"]
        .iter()
        .collect();
    let mut meta: Value = serde_json::from_reader(File::open(&file)?)?;

    // Make sure we catch the validation failure.
    match Distribution::try_from(meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert!(e.to_string().contains(" missing properties 'version")),
    }

    // Make sure we fail on invalid version.
    match Distribution::from_version(99, meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert_eq!("cannot determine meta-spec version", e.to_string()),
    }

    // Should fail when no meta-spec.
    meta.as_object_mut().unwrap().remove("meta-spec");
    match Distribution::try_from(meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert_eq!("cannot determine meta-spec version", e.to_string()),
    }

    // Make sure we catch a failure parsing into a Distribution struct.
    match v2::from_value(json!({"invalid": true})) {
        Ok(_) => panic!("Should have failed on invalid meta contents but did not",),
        Err(e) => assert_eq!("missing field `name`", e.to_string()),
    }

    Ok(())
}

#[test]
fn test_try_merge_v1() -> Result<(), Error> {
    // Load a v1 META file.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let widget_file = dir.join("v1").join("widget.json");
    let contents: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    // expect maps a JSON pointer to an expected value.
    for (name, patches, expect) in [
        (
            "license",
            vec![json!({"license": "MIT"})],
            json!({"/license": "MIT"}),
        ),
        (
            "tle",
            vec![json!({"contents": {"extensions": {"widget": {"tle": true}}}})],
            json!({"/contents/extensions/widget": {
                "control": "widget.control",
                "sql": "sql/widget.sql.in",
                "tle": true,
            }}),
        ),
        (
            "categories",
            vec![json!({"classifications": {"categories": ["Analytics", "Connectors"]}})],
            json!({"/classifications/categories": ["Analytics", "Connectors"]}),
        ),
        (
            "tags",
            vec![json!({"classifications": {"tags": ["hi", "go", "ick"]}})],
            json!({"/classifications/tags": ["hi", "go", "ick"]}),
        ),
        (
            "resources",
            vec![json!({"resources": {
                "issues": "https://example.com/issues",
                "repository": "https://example.com/repo",
            }})],
            json!({"/resources": {
                "homepage": "http://widget.example.org/",
                "issues": "https://example.com/issues",
                "repository": "https://example.com/repo",
            }}),
        ),
        (
            "delete packages",
            vec![json!({"dependencies": {"packages": null}})],
            json!({"/dependencies": {"postgres": { "version": "8.0.0" }}}),
        ),
    ] {
        run_merge_case(name, &contents, patches.as_slice(), &expect)?;
    }

    Ok(())
}

#[test]
fn test_try_merge_v2() -> Result<(), Error> {
    // Load a v2 META file.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let widget_file = dir.join("v2").join("minimal.json");
    let contents: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    // expect maps a JSON pointer to an expected value.
    for (name, patches, expect) in [
        (
            "license",
            vec![json!({"license": "MIT"})],
            json!({"/license": "MIT"}),
        ),
        (
            "tle",
            vec![json!({"contents": {"extensions": {"pair": {"tle": true}}}})],
            json!({"/contents/extensions/pair": {
                "sql": "sql/pair.sql",
                "control": "pair.control",
                "tle": true,
            }}),
        ),
        (
            "multiple patches",
            vec![
                json!({"license": "MIT"}),
                json!({"classifications": {"categories": ["Analytics", "Connectors"]}}),
            ],
            json!({
                "/license": "MIT",
                "/classifications/categories": ["Analytics", "Connectors"],
            }),
        ),
    ] {
        run_merge_case(name, &contents, patches.as_slice(), &expect)?;
    }

    Ok(())
}

fn run_merge_case(
    name: &str,
    orig: &Value,
    patches: &[Value],
    expect: &Value,
) -> Result<(), Error> {
    let mut meta = vec![orig.clone()];
    for p in patches {
        meta.push(p.clone());
    }
    match Distribution::try_from(meta.as_slice()) {
        Err(e) => panic!("patching {name} failed: {e}"),
        Ok(dist) => {
            // Convert the Distribution object to JSON.
            let output: Result<Value, Error> = dist.try_into();
            match output {
                Err(e) => panic!("{name} serialization failed: {e}"),
                Ok(val) => {
                    // Compare expected values at pointers.
                    for (p, v) in expect.as_object().unwrap() {
                        assert_eq!(v, val.pointer(p).unwrap())
                    }
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_try_merge_err() -> Result<(), Error> {
    // Load invalid meta.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let widget_file = dir.join("invalid.json");
    let invalid: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    let empty = json!({});
    let bad_version = json!({"meta-spec": { "version": null}});

    for (name, arg, err) in [
        ("no meta", vec![], "meta contains no values"),
        (
            "no version",
            vec![empty],
            "cannot determine meta-spec version",
        ),
        (
            "bad version",
            vec![bad_version],
            "cannot determine meta-spec version",
        ),
        (
            "invalid",
            vec![invalid],
            "jsonschema validation failed with https://pgxn.org/meta/v2/distribution.schema.json#\n- at '': missing properties 'version'",
        ),
    ] {
        match Distribution::try_from(arg.as_slice()) {
            Ok(_) => panic!("patching {name} unexpectedly succeeded"),
            Err(e) => assert!(e.to_string().contains(err), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn test_try_merge_partman() -> Result<(), Error> {
    // Test the real-world pg_partman JSON with a patch to build the expected
    // v2 JSON. First, load the original metadata.
    let original_meta = json!({
        "name": "pg_partman",
        "abstract": "Extension to manage partitioned tables by time or ID",
        "version": "5.1.0",
        "maintainer": [
            "Keith Fiske <keith@keithf4.com>"
        ],
        "license": "postgresql",
        "generated_by": "Keith Fiske",
        "release_status": "stable",
        "prereqs": {
            "runtime": {
                "requires": {
                    "PostgreSQL": "14.0"
                },
                "recommends": {
                    "pg_jobmon": "1.4.1"
                }
            }
        },
        "provides": {
            "pg_partman": {
                "file": "sql/pg_partman--5.1.0.sql",
                "docfile": "doc/pg_partman.md",
                "version": "5.1.0",
                "abstract": "Extension to manage partitioned tables by time or ID"
            }
        },
        "resources": {
            "bugtracker": {
                "web": "https://github.com/pgpartman/pg_partman/issues"
            },
            "repository": {
                "url": "git://github.com/pgpartman/pg_partman.git" ,
                "web": "https://github.com/pgpartman/pg_partman",
                "type": "git"
            }
        },
        "meta-spec": {
          "version": "1.0.0",
          "url": "http://pgxn.org/meta/spec.txt"
        },
        "tags": [
            "partition",
            "partitions",
            "partitioning",
            "table",
            "tables",
            "bgw",
            "background worker",
            "custom background worker"
        ]
    });

    // Load expected metadata.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "v2"]
        .iter()
        .collect();
    let widget_file = dir.join("pg_partman.json");
    let expect_meta: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    // Create the patch.
    let patch = json!({
        "contents": {
          "extensions": {
            "pg_partman": {
              "sql": "sql/types/types.sql",
            }
          },
          "modules": {
            "pg_partman_bgw": {
              "type": "bgw",
              "lib": "src/pg_partman_bgw",
              "preload": "server"
            }
          },
          "apps": {
            "check_unique_constraint": {
              "lang": "python",
              "bin": "bin/common/check_unique_constraint.py",
              "abstract": "Check that all rows in a partition set are unique for the given columns"
            },
            "dump_partition": {
              "lang": "python",
              "bin": "bin/common/dump_partition.py",
              "abstract": "Dump out and then drop all tables contained in a schema."
            },
            "vacuum_maintenance": {
              "lang": "python",
              "bin": "bin/common/vacuum_maintenance.py",
              "abstract": "Performing vacuum maintenance on to avoid excess vacuuming and transaction id wraparound issues"
            }
          }
        },
        "producer": "David E. Wheeler",
        "resources": {
          "issues": "https://github.com/theory/pg-envvar/issues/",
          "repository": "https://github.com/theory/pg-envvar/",
          "badges": [
            {
              "alt": "CI Status",
              "src": "https://github.com/theory/pg-envvar/actions/workflows/ci.yml/badge.svg",
              "url": "https://github.com/theory/pg-envvar/actions/workflows/ci.yml"
            }
          ]
        },
        "dependencies": {
          "postgres": {
            "version": "14.0"
          },
          "packages": {
            "run": {
              "requires": {
                "pkg:generic/python": "2.0",
                "pkg:pypi/psycopg2": 0
              },
            }
          }
        },
        "classifications": {
          "categories": ["Orchestration"],
        },
    });

    // Apply the patch.
    let meta = [original_meta, patch];
    match Distribution::try_from(&meta[..]) {
        Err(e) => panic!("patching part man failed: {e}"),
        Ok(dist) => {
            // Convert the Distributions object to JSON.
            let output: Result<Value, Error> = dist.try_into();
            match output {
                Err(e) => panic!("partman serialization failed: {e}"),
                Ok(val) => {
                    // Compare to expected.
                    assert_json_diff::assert_json_eq!(&expect_meta, &val);
                }
            }
        }
    }

    Ok(())
}

#[test]
fn test_spec() {
    for (name, json) in [
        (
            "both",
            json!({"version": "2.0.0", "url": "https://example.com"}),
        ),
        ("version only", json!({"version": "2.0.4"})),
    ] {
        let spec: Spec = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("version").unwrap().as_str().unwrap(),
            spec.version().to_string(),
            "{name} version",
        );
        match json.get("url") {
            None => assert!(spec.url().is_none(), "{name} url"),
            Some(url) => assert_eq!(url.as_str().unwrap(), spec.url().unwrap(), "{name} url"),
        }
    }
}

#[test]
fn test_maintainer() {
    for (name, json) in [
        (
            "all fields",
            json!({
                "name": "Barrack Obama",
                "email": "potus@example.com",
                "url":  "https://potus.example.com",
                "x_foo": true,
                "X_hi": [5, 6, 7],
            }),
        ),
        (
            "name and email",
            json!({
                "name": "Barrack Obama",
                "email": "potus@example.com",
            }),
        ),
        (
            "name and url",
            json!({
                "name": "Barrack Obama",
                "url":  "https://potus.example.com",
            }),
        ),
    ] {
        let maintainer: Maintainer = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("name").unwrap().as_str().unwrap(),
            maintainer.name().to_string(),
            "{name} name",
        );
        match json.get("email") {
            None => assert!(maintainer.email().is_none(), "{name} email"),
            Some(email) => assert_eq!(
                email.as_str().unwrap(),
                maintainer.email().unwrap(),
                "{name} email"
            ),
        }
        match json.get("url") {
            None => assert!(maintainer.url().is_none(), "{name} url"),
            Some(url) => assert_eq!(
                url.as_str().unwrap(),
                maintainer.url().unwrap(),
                "{name} url"
            ),
        }
        assert_eq!(
            &exes_from(&json),
            maintainer.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_extension() {
    for (name, json) in [
        (
            "all fields",
            json!({
                "control": "pair.control",
                "abstract": "We have assumed control",
                "tle": true,
                "sql": "pair.sql",
                "doc": "doc/pair.md",
                "x_foo": "hello",
                "X_zzz": {"yes": true},
            }),
        ),
        (
            "minimal",
            json!({
                "control": "pair.control",
                "sql": "pair.sql",
            }),
        ),
        (
            "false tle",
            json!({
                "control": "pair.control",
                "tle": false,
                "sql": "pair.sql",
            }),
        ),
    ] {
        let extension: Extension = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("control").unwrap().as_str().unwrap(),
            extension.control().to_string(),
            "{name} control",
        );
        assert_eq!(
            json.get("sql").unwrap().as_str().unwrap(),
            extension.sql().to_string(),
            "{name} sql",
        );
        let tle = match json.get("tle") {
            None => false,
            Some(tle) => tle.as_bool().unwrap(),
        };
        assert_eq!(tle, extension.tle());
        match json.get("abstract") {
            None => assert!(extension.abs_tract().is_none(), "{name} abstract"),
            Some(abs) => assert_eq!(
                abs.as_str().unwrap(),
                extension.abs_tract().unwrap(),
                "{name} abstract"
            ),
        }
        match json.get("doc") {
            None => assert!(extension.doc().is_none(), "{name} doc"),
            Some(doc) => assert_eq!(
                doc.as_str().unwrap(),
                extension.doc().unwrap(),
                "{name} doc"
            ),
        }
        assert_eq!(
            &exes_from(&json),
            extension.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_module_type() {
    for (name, mod_type) in [
        ("extension", ModuleType::Extension),
        ("hook", ModuleType::Hook),
        ("bgw", ModuleType::Bgw),
    ] {
        let mt: ModuleType = serde_json::from_value(json!(name)).unwrap();
        assert_eq!(mod_type, mt);
        assert_eq!(name, mt.to_string())
    }
}

#[test]
fn test_preload() {
    for (name, preload) in [("server", Preload::Server), ("session", Preload::Session)] {
        let pre: Preload = serde_json::from_value(json!(name)).unwrap();
        assert_eq!(preload, pre);
        assert_eq!(name, pre.to_string())
    }
}

#[test]
fn test_module() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "type": "hook",
              "lib": "lib/my_hook",
              "doc": "doc/my_hook.md",
              "preload": "session",
              "abstract": "My hook",
              "x_foo": 42,
              "X_YZ": 98.6,
            }),
        ),
        (
            "minimal",
            json!({
              "type": "extension",
              "lib": "lib/my_hook",
            }),
        ),
    ] {
        let module: Module = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("type").unwrap().as_str().unwrap(),
            module.kind().to_string(),
            "{name} type",
        );
        assert_eq!(
            json.get("lib").unwrap().as_str().unwrap(),
            module.lib().to_string(),
            "{name} lib",
        );
        match json.get("preload") {
            None => assert!(module.preload().is_none(), "{name} preload"),
            Some(pre) => assert_eq!(
                pre.as_str().unwrap(),
                module.preload().unwrap().to_string(),
                "{name} preload"
            ),
        }
        match json.get("abstract") {
            None => assert!(module.abs_tract().is_none(), "{name} abstract"),
            Some(abs) => assert_eq!(
                abs.as_str().unwrap(),
                module.abs_tract().unwrap(),
                "{name} abstract"
            ),
        }
        match json.get("doc") {
            None => assert!(module.doc().is_none(), "{name} doc"),
            Some(doc) => assert_eq!(doc.as_str().unwrap(), module.doc().unwrap(), "{name} doc"),
        }
        assert_eq!(
            &exes_from(&json),
            module.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_app() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "lang": "c",
              "bin": "bin/my_hook",
              "lib": "lib/my_hook",
              "doc": "doc/my_hook.md",
              "man": "doc/my_hoo.man",
              "html": "doc/html",
              "abstract": "My hook",
              "x_foo": [1, 2, 3],
              "X_XXX": "XXX",
            }),
        ),
        ("minimal", json!({"bin": "bin/my_hook"})),
    ] {
        let app: App = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("bin").unwrap().as_str().unwrap(),
            app.bin().to_string(),
            "{name} bin",
        );
        match json.get("lang") {
            None => assert!(app.lang().is_none(), "{name} lang"),
            Some(lang) => assert_eq!(
                lang.as_str().unwrap(),
                app.lang().unwrap().to_string(),
                "{name} lang"
            ),
        }
        match json.get("abstract") {
            None => assert!(app.abs_tract().is_none(), "{name} abstract"),
            Some(abs) => assert_eq!(
                abs.as_str().unwrap(),
                app.abs_tract().unwrap(),
                "{name} abstract"
            ),
        }
        match json.get("doc") {
            None => assert!(app.doc().is_none(), "{name} doc"),
            Some(doc) => assert_eq!(doc.as_str().unwrap(), app.doc().unwrap(), "{name} doc"),
        }
        match json.get("lib") {
            None => assert!(app.lib().is_none(), "{name} lib"),
            Some(lib) => assert_eq!(lib.as_str().unwrap(), app.lib().unwrap(), "{name} lib"),
        }
        match json.get("man") {
            None => assert!(app.man().is_none(), "{name} man"),
            Some(man) => assert_eq!(man.as_str().unwrap(), app.man().unwrap(), "{name} man"),
        }
        match json.get("html") {
            None => assert!(app.html().is_none(), "{name} html"),
            Some(html) => assert_eq!(html.as_str().unwrap(), app.html().unwrap(), "{name} html"),
        }
        assert_eq!(&exes_from(&json), app.custom_props(), "{name} custom_props");
    }
}

#[test]
fn test_contents() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "extensions": {"pair": {"control": "pair.control", "sql": "pair.sql"}},
              "modules": {"pair": {"type": "bgw", "lib": "lib/pair"}},
              "apps": {"thing": { "bin": "bin/thing "}},
              "x_foo": {"hi": 42},
              "X_1234": null,
            }),
        ),
        (
            "just extensions",
            json!({
              "extensions": {"pair": {"control": "pair.control", "sql": "pair.sql"}},
            }),
        ),
        (
            "all modules",
            json!({
              "modules": {"pair": {"type": "bgw", "lib": "lib/pair"}},
            }),
        ),
        (
            "just apps",
            json!({ "apps": {"thing": { "bin": "bin/thing "}}}),
        ),
    ] {
        let contents: Contents = serde_json::from_value(json.clone()).unwrap();
        match json.get("extensions") {
            None => assert!(contents.extensions().is_none(), "{name} extensions"),
            Some(e) => {
                let ext: HashMap<String, Extension> = serde_json::from_value(e.clone()).unwrap();
                assert_eq!(&ext, contents.extensions().unwrap(), "{name} extensions");
            }
        }
        match json.get("modules") {
            None => assert!(contents.modules().is_none(), "{name} modules"),
            Some(m) => {
                let modules: HashMap<String, Module> = serde_json::from_value(m.clone()).unwrap();
                assert_eq!(&modules, contents.modules().unwrap(), "{name} modules");
            }
        }
        match json.get("apps") {
            None => assert!(contents.apps().is_none(), "{name} apps"),
            Some(a) => {
                let apps: HashMap<String, App> = serde_json::from_value(a.clone()).unwrap();
                assert_eq!(&apps, contents.apps().unwrap(), "{name} apps");
            }
        }
        assert_eq!(
            &exes_from(&json),
            contents.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_classifications() {
    for (name, json) in [
        (
            "tags and categories",
            json!({
              "tags": ["a", "b", "c"],
              "categories": ["x", "y", "z"],
              "x_foo": null,
            }),
        ),
        ("just tags", json!({"tags": ["a", "b", "c"]})),
        ("just categories", json!({"categories": ["x", "y", "z"]})),
    ] {
        let classes: Classifications = serde_json::from_value(json.clone()).unwrap();
        match json.get("tags") {
            None => assert!(classes.tags().is_none(), "{name} tags"),
            Some(tags) => {
                let tags: Vec<String> = serde_json::from_value(tags.clone()).unwrap();
                assert_eq!(&tags, classes.tags().unwrap(), "{name} tags");
            }
        }
        match json.get("categories") {
            None => assert!(classes.categories().is_none(), "{name} categories"),
            Some(cats) => {
                let cats: Vec<String> = serde_json::from_value(cats.clone()).unwrap();
                assert_eq!(&cats, classes.categories().unwrap(), "{name} categories");
            }
        }
        assert_eq!(
            &exes_from(&json),
            classes.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_postgres() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "version": ">= 12.0, < 17.0",
              "with": ["xml", "uuid", "perl"],
              "x_foo": false,
              "X_xyz": 42,
            }),
        ),
        ("just version", json!({"version": "12"})),
    ] {
        let pg: Postgres = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("version").unwrap().as_str().unwrap(),
            pg.version(),
            "{name} version",
        );
        match json.get("with") {
            None => assert!(pg.with().is_none(), "{name} with"),
            Some(with) => {
                let with: Vec<String> = serde_json::from_value(with.clone()).unwrap();
                assert_eq!(&with, pg.with().unwrap(), "{name} with");
            }
        }
        assert_eq!(&exes_from(&json), pg.custom_props(), "{name} custom_props");
    }
}

#[test]
fn test_pipeline() {
    for (name, pipeline) in [
        ("pgxs", Pipeline::Pgxs),
        ("meson", Pipeline::Meson),
        ("pgrx", Pipeline::Pgrx),
        ("autoconf", Pipeline::Autoconf),
        ("cmake", Pipeline::Cmake),
    ] {
        let pipe: Pipeline = serde_json::from_value(json!(name)).unwrap();
        assert_eq!(pipeline, pipe);
        assert_eq!(name, pipe.to_string())
    }
}

#[test]
fn test_version_range() {
    for (name, json, exp, str) in [
        ("integer", json!(0), VersionRange::Integer(0), "0"),
        (
            "string",
            json!("12"),
            VersionRange::String("12".to_string()),
            "12",
        ),
    ] {
        let range: VersionRange = serde_json::from_value(json!(json)).unwrap();
        assert_eq!(exp, range, "{name} enum");
        assert_eq!(str, range.to_string(), "{name} string")
    }
}

#[test]
fn test_phase() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "requires": {"pkg:pgxn/pair": 0},
              "recommends": {"pkg:pgxn/pair": "0.5"},
              "suggests": {"pkg:pgxn/pair": "1.0.0"},
              "conflicts": {"pkg:pgxn/tricorn": 0},
              "x_foo": "hi",
            }),
        ),
        ("just requires", json!({"requires": {"pkg:pgxn/pair": 0}})),
        (
            "just recommends",
            json!({"recommends": {"pkg:pgxn/pair": 0}}),
        ),
        ("just suggests", json!({"suggests": {"pkg:pgxn/pair": 0}})),
        ("just conflicts", json!({"conflicts": {"pkg:pgxn/pair": 0}})),
    ] {
        let phase: Phase = serde_json::from_value(json.clone()).unwrap();
        match json.get("requires") {
            None => assert!(phase.requires().is_none(), "{name} requires"),
            Some(p) => {
                let p: HashMap<String, VersionRange> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, phase.requires().unwrap(), "{name} requires");
            }
        }
        match json.get("recommends") {
            None => assert!(phase.recommends().is_none(), "{name} recommends"),
            Some(p) => {
                let p: HashMap<String, VersionRange> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, phase.recommends().unwrap(), "{name} recommends");
            }
        }
        match json.get("suggests") {
            None => assert!(phase.suggests().is_none(), "{name} suggests"),
            Some(p) => {
                let p: HashMap<String, VersionRange> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, phase.suggests().unwrap(), "{name} suggests");
            }
        }
        match json.get("conflicts") {
            None => assert!(phase.conflicts().is_none(), "{name} conflicts"),
            Some(p) => {
                let p: HashMap<String, VersionRange> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, phase.conflicts().unwrap(), "{name} conflicts");
            }
        }
        assert_eq!(
            &exes_from(&json),
            phase.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_packages() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "configure": {"requires": {"pkg:pgxn/pair": 0}},
              "build": {"recommends": {"pkg:pgxn/pair": "0.5"}},
              "test": {"suggests": {"pkg:pgxn/pair": "1.0.0"}},
              "run": {"conflicts": {"pkg:pgxn/tricorn": 0}},
              "develop": {"conflicts": {"pkg:pgxn/tricorn": 0}},
              "x_foo": {"requires": {"pkg:postgres/pl": 0}},
            }),
        ),
        (
            "just configure",
            json!({"configure": {"requires": {"pkg:pgxn/pair": 0}}}),
        ),
        (
            "just build",
            json!({"build": {"requires": {"pkg:pgxn/pair": 0}}}),
        ),
        (
            "just test",
            json!({"test": {"requires": {"pkg:pgxn/pair": 0}}}),
        ),
        (
            "just run",
            json!({"run": {"requires": {"pkg:pgxn/pair": 0}}}),
        ),
        (
            "just develop",
            json!({"develop": {"requires": {"pkg:pgxn/pair": 0}}}),
        ),
    ] {
        let pkgs: Packages = serde_json::from_value(json.clone()).unwrap();
        match json.get("configure") {
            None => assert!(pkgs.configure().is_none(), "{name} configure"),
            Some(p) => {
                let p: Phase = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, pkgs.configure().unwrap(), "{name} configure");
            }
        }
        match json.get("build") {
            None => assert!(pkgs.build().is_none(), "{name} build"),
            Some(p) => {
                let p: Phase = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, pkgs.build().unwrap(), "{name} build");
            }
        }
        match json.get("test") {
            None => assert!(pkgs.test().is_none(), "{name} test"),
            Some(p) => {
                let p: Phase = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, pkgs.test().unwrap(), "{name} test");
            }
        }
        match json.get("run") {
            None => assert!(pkgs.run().is_none(), "{name} run"),
            Some(p) => {
                let p: Phase = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, pkgs.run().unwrap(), "{name} run");
            }
        }
        match json.get("develop") {
            None => assert!(pkgs.develop().is_none(), "{name} develop"),
            Some(p) => {
                let p: Phase = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, pkgs.develop().unwrap(), "{name} develop");
            }
        }
        assert_eq!(
            &exes_from(&json),
            pkgs.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_variations() {
    for (name, json) in [
        (
            "customize packages",
            json!({
              "x_foo": "hi",
              "where": { "platforms": ["linux"] },
              "dependencies": {
                "packages": {
                  "configure": {
                    "requires": {
                      "pkg:generic/fork": 0
                    }
                  }
                }
              }
            }),
        ),
        (
            "customize postgres",
            json!({
              "where": { "postgres": { "version": ">= 16.0" } },
              "dependencies": {
                "postgres": { "version": ">= 16.0", "with": ["zstd"] }
              }
            }),
        ),
    ] {
        let vars: Variations = serde_json::from_value(json.clone()).unwrap();
        let d: Dependencies = serde_json::from_value(json.get("where").unwrap().clone()).unwrap();
        assert_eq!(&d, vars.wheres(), "{name} where");

        let d: Dependencies =
            serde_json::from_value(json.get("dependencies").unwrap().clone()).unwrap();
        assert_eq!(&d, vars.dependencies(), "{name} dependencies");
        assert_eq!(
            &exes_from(&json),
            vars.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_dependencies() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "platforms": ["linux", "macOS"],
              "postgres": { "version": "14.0" },
              "pipeline": "pgxs",
              "packages": {
                "configure": {
                  "requires": {
                    "pkg:generic/fork": 0
                  },
                },
              },
              "variations": [{
                "where": { "postgres": { "version": ">= 16.0" } },
                "dependencies": {
                  "postgres": { "version": ">= 16.0", "with": ["zstd"] }
                },
              }],
              "x_foo": 42,
            }),
        ),
        ("just platforms", json!({"platforms": ["linux", "macOS"]})),
        ("just postgres", json!({"postgres": { "version": "14.0" }})),
        ("just pipeline", json!({"pipeline": "pgrx"})),
        (
            "just packages",
            json!({
              "packages": {
                "configure": {
                  "requires": {
                    "pkg:generic/fork": 0
                  }
                }
              },
            }),
        ),
        (
            "just variations",
            json!({
              "variations": [{
                "where": { "postgres": { "version": ">= 16.0" } },
                "dependencies": {
                  "postgres": { "version": ">= 16.0", "with": ["zstd"] }
                },
              }],
            }),
        ),
    ] {
        let deps: Dependencies = serde_json::from_value(json.clone()).unwrap();
        match json.get("platforms") {
            None => assert!(deps.platforms().is_none(), "{name} platforms"),
            Some(p) => {
                let p: Vec<String> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, deps.platforms().unwrap(), "{name} platforms");
            }
        }
        match json.get("postgres") {
            None => assert!(deps.postgres().is_none(), "{name} postgres"),
            Some(p) => {
                let p: Postgres = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, deps.postgres().unwrap(), "{name} postgres");
            }
        }
        match json.get("pipeline") {
            None => assert!(deps.pipeline().is_none(), "{name} pipeline"),
            Some(p) => {
                let p: Pipeline = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, deps.pipeline().unwrap(), "{name} pipeline");
            }
        }
        match json.get("packages") {
            None => assert!(deps.packages().is_none(), "{name} packages"),
            Some(p) => {
                let p: Packages = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, deps.packages().unwrap(), "{name} packages");
            }
        }
        match json.get("variations") {
            None => assert!(deps.variations().is_none(), "{name} variations"),
            Some(p) => {
                let p: Vec<Variations> = serde_json::from_value(p.clone()).unwrap();
                assert_eq!(&p, deps.variations().unwrap(), "{name} variations");
            }
        }
        assert_eq!(
            &exes_from(&json),
            deps.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_badge() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "alt": "Test Status",
              "src": "https://test.packages.postgresql.org/github.com/example/pair.svg",
              "url": "https://test.packages.postgresql.org/github.com/example/pair.html",
              "x_foo": true,
            }),
        ),
        (
            "no url",
            json!({
              "alt": "Test Status",
              "src": "https://test.packages.postgresql.org/github.com/example/pair.svg",
            }),
        ),
    ] {
        let badge: Badge = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("src").unwrap().as_str().unwrap(),
            badge.src().to_string(),
            "{name} src",
        );
        assert_eq!(
            json.get("alt").unwrap().as_str().unwrap(),
            badge.alt().to_string(),
            "{name} alt",
        );
        match json.get("url") {
            None => assert!(badge.url().is_none(), "{name} url"),
            Some(url) => assert_eq!(url.as_str().unwrap(), badge.url().unwrap(), "{name} url"),
        }
        assert_eq!(
            &exes_from(&json),
            badge.custom_props(),
            "{name} custom_props"
        );
    }
}

#[test]
fn test_resources() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "homepage": "https://pair.example.com",
              "issues": "https://github.com/example/pair/issues",
              "docs": "https://pair.example.com/docs",
              "support": "https://github.com/example/pair/discussions",
              "repository": "https://github.com/example/pair",
              "badges": [
                {
                  "alt": "Test Status",
                  "src": "https://test.packages.postgresql.org/github.com/example/pair.svg"
                }
              ],
              "x_foo": 42,
            }),
        ),
        (
            "just homepage",
            json!({"homepage": "https://pair.example.com"}),
        ),
    ] {
        let res: Resources = serde_json::from_value(json.clone()).unwrap();
        match json.get("homepage") {
            None => assert!(res.homepage().is_none(), "{name} homepage"),
            Some(url) => assert_eq!(
                url.as_str().unwrap(),
                res.homepage().unwrap(),
                "{name} homepage"
            ),
        }
        match json.get("issues") {
            None => assert!(res.issues().is_none(), "{name} issues"),
            Some(url) => assert_eq!(
                url.as_str().unwrap(),
                res.issues().unwrap(),
                "{name} issues"
            ),
        }
        match json.get("docs") {
            None => assert!(res.docs().is_none(), "{name} docs"),
            Some(url) => assert_eq!(url.as_str().unwrap(), res.docs().unwrap(), "{name} docs"),
        }
        match json.get("support") {
            None => assert!(res.support().is_none(), "{name} support"),
            Some(url) => assert_eq!(
                url.as_str().unwrap(),
                res.support().unwrap(),
                "{name} support"
            ),
        }
        match json.get("repository") {
            None => assert!(res.repository().is_none(), "{name} repository"),
            Some(url) => assert_eq!(
                url.as_str().unwrap(),
                res.repository().unwrap(),
                "{name} repository"
            ),
        }
        match json.get("badges") {
            None => assert!(res.badges().is_none(), "{name} badges"),
            Some(b) => {
                let p: Vec<Badge> = serde_json::from_value(b.clone()).unwrap();
                assert_eq!(&p, res.badges().unwrap(), "{name} badges");
            }
        }
        assert_eq!(&exes_from(&json), res.custom_props(), "{name} custom_props");
    }
}

#[test]
fn test_artifact() {
    for (name, json) in [
        (
            "all fields",
            json!({
              "type": "rpm",
              "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0.deb",
              "platform": "linux-arm64",
              "sha256": "a97ab886f7d7989d559f8cc3fcb655e4c9056c33045a935803c85cd1bd38b327",
              "sha512": "51ec2ca2366d01de37d7f02d7aa9d38fa60abad31eebd68e6af0b8e39c31323b91284cd127eed54c4bddf75f8698ee316ee6ebca8fd012658f19f3166554874a",
              "x_foo": "go",
            }),
        ),
        (
            "source",
            json!({
              "type": "source",
              "url": "https://github.com/theory/pg-pair/releases/download/v1.1.0/pair-1.1.0.zip",
              "sha256": "2b9d2416096d2930be51e5332b70bcd97846947777a93e4a3d65fe1b5fd7b004",
            }),
        ),
    ] {
        let art: Artifact = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(
            json.get("url").unwrap().as_str().unwrap(),
            art.url().to_string(),
            "{name} url",
        );
        assert_eq!(
            json.get("type").unwrap().as_str().unwrap(),
            art.kind().to_string(),
            "{name} type",
        );
        match json.get("platform") {
            None => assert!(art.platform().is_none(), "{name} platform"),
            Some(platform) => assert_eq!(
                platform.as_str().unwrap(),
                art.platform().unwrap(),
                "{name} platform"
            ),
        }
        match json.get("sha256") {
            None => assert!(art.sha256().is_none(), "{name} sha256"),
            Some(sha256) => assert_eq!(
                sha256.as_str().unwrap(),
                art.sha256().unwrap(),
                "{name} sha256"
            ),
        }
        match json.get("sha512") {
            None => assert!(art.sha512().is_none(), "{name} sha512"),
            Some(sha512) => assert_eq!(
                sha512.as_str().unwrap(),
                art.sha512().unwrap(),
                "{name} sha512"
            ),
        }
        assert_eq!(&exes_from(&json), art.custom_props(), "{name} custom_props");
    }
}

#[test]
fn test_distribution() -> Result<(), Error> {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "v2"]
        .iter()
        .collect();
    let glob = Glob::new("*.json")?;

    for path in glob.walk(dir) {
        let path = path?.into_path();
        let name = path.as_path().to_str().unwrap();
        let contents: Value = serde_json::from_reader(File::open(&path)?)?;

        match Distribution::load(&path) {
            Err(e) => panic!("{name} failed: {e}"),
            Ok(dist) => {
                // Required fields.
                assert_eq!(
                    contents.get("name").unwrap().as_str().unwrap(),
                    dist.name(),
                    "{name} name",
                );
                assert_eq!(
                    contents.get("version").unwrap().as_str().unwrap(),
                    dist.version().to_string(),
                    "{name} version",
                );
                assert_eq!(
                    contents.get("abstract").unwrap().as_str().unwrap(),
                    dist.abs_tract().to_string(),
                    "{name} abstract",
                );
                assert_eq!(
                    contents.get("license").unwrap().as_str().unwrap(),
                    dist.license(),
                    "{name} license",
                );

                let val: Spec =
                    serde_json::from_value(contents.get("meta-spec").unwrap().clone()).unwrap();
                assert_eq!(&val, dist.spec(), "{name} spec");

                let val: Vec<Maintainer> =
                    serde_json::from_value(contents.get("maintainers").unwrap().clone()).unwrap();
                assert_eq!(&val, dist.maintainers(), "{name} maintainers");

                let val: Contents =
                    serde_json::from_value(contents.get("contents").unwrap().clone()).unwrap();
                assert_eq!(&val, dist.contents(), "{name} contents");

                // Optional fields.
                match contents.get("description") {
                    None => assert!(dist.description().is_none(), "{name} description"),
                    Some(description) => assert_eq!(
                        description.as_str().unwrap(),
                        dist.description().unwrap(),
                        "{name} description"
                    ),
                }
                match contents.get("producer") {
                    None => assert!(dist.producer().is_none(), "{name} producer"),
                    Some(producer) => assert_eq!(
                        producer.as_str().unwrap(),
                        dist.producer().unwrap(),
                        "{name} producer"
                    ),
                }
                match contents.get("classifications") {
                    None => assert!(dist.classifications().is_none(), "{name} classifications"),
                    Some(val) => {
                        let p: Classifications = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(
                            &p,
                            dist.classifications().unwrap(),
                            "{name} classifications"
                        );
                    }
                }
                match contents.get("ignore") {
                    None => assert!(dist.ignore().is_none(), "{name} ignore"),
                    Some(val) => {
                        let p: Vec<String> = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, dist.ignore().unwrap(), "{name} ignore");
                    }
                }
                match contents.get("dependencies") {
                    None => assert!(dist.dependencies().is_none(), "{name} dependencies"),
                    Some(val) => {
                        let p: Dependencies = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, dist.dependencies().unwrap(), "{name} dependencies");
                    }
                }
                match contents.get("resources") {
                    None => assert!(dist.resources().is_none(), "{name} resources"),
                    Some(val) => {
                        let p: Resources = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, dist.resources().unwrap(), "{name} resources");
                    }
                }
                match contents.get("artifacts") {
                    None => assert!(dist.artifacts().is_none(), "{name} artifacts"),
                    Some(val) => {
                        let p: Vec<Artifact> = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, dist.artifacts().unwrap(), "{name} artifacts");
                    }
                }
                assert_eq!(
                    &exes_from(&contents),
                    dist.custom_props(),
                    "{name} custom_props"
                );
            }
        }
    }

    Ok(())
}

/// Extracts the subset of val (which must be an instance of Value::Object)
/// where the property names start with `x_` or `X_`. Used for testing
/// custom_props.
fn exes_from(val: &Value) -> HashMap<String, Value> {
    val.as_object()
        .unwrap()
        .into_iter()
        .filter(|(key, _)| key.starts_with("x_") || key.starts_with("X_"))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}
