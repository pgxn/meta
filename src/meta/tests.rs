use super::*;
use serde_json::{json, Value};
use std::{error::Error, fs, fs::File, path::PathBuf};
use wax::Glob;

#[test]
fn test_corpus() -> Result<(), Box<dyn Error>> {
    for v_dir in ["v1", "v2"] {
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", v_dir]
            .iter()
            .collect();
        let glob = Glob::new("*.json")?;

        for path in glob.walk(dir) {
            let path = path?.into_path();
            let contents: Value = serde_json::from_reader(File::open(&path)?)?;

            // Test try_from path.
            if let Err(e) = Meta::try_from(&path) {
                panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap());
            }

            // Test try_from str.
            let str: String = fs::read_to_string(&path)?;
            match Meta::try_from(&str) {
                Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                Ok(m) => {
                    if v_dir == "v2" {
                        // Make sure round-trip produces the same JSON.
                        let output: Result<Value, Box<dyn Error>> = m.try_into();
                        match output {
                            Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                            Ok(val) => {
                                assert_json_diff::assert_json_eq!(&contents, &val);
                            }
                        };
                    }
                }
            }

            // Test try_from value.
            match Meta::try_from(contents.clone()) {
                Err(e) => panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap()),
                Ok(m) => {
                    if v_dir == "v2" {
                        // Make sure value round-trip produces the same JSON.
                        let output: Result<String, Box<dyn Error>> = m.try_into();
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

            println!("Example {v_dir}/{:?} ok", path.file_name().unwrap());
        }
    }
    Ok(())
}

#[test]
fn test_bad_corpus() -> Result<(), Box<dyn Error>> {
    let file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "invalid.json"]
        .iter()
        .collect();
    let mut meta: Value = serde_json::from_reader(File::open(&file)?)?;

    // Make sure we catch the validation failure.
    match Meta::try_from(meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert!(e.to_string().contains(" missing properties 'version")),
    }

    // Make sure we fail on invalid version.
    match Meta::from_version(99, meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert_eq!("Unknown meta version 99", e.to_string()),
    }

    // Should fail when no meta-spec.
    meta.as_object_mut().unwrap().remove("meta-spec");
    match Meta::try_from(meta.clone()) {
        Ok(_) => panic!(
            "Should have failed on {:?} but did not",
            file.file_name().unwrap()
        ),
        Err(e) => assert_eq!("Cannot determine meta-spec version", e.to_string()),
    }

    // Make sure we catch a failure parsing into a Meta struct.
    match v2::from_value(json!({"invalid": true})) {
        Ok(_) => panic!("Should have failed on invalid meta contents but did not",),
        Err(e) => assert_eq!("missing field `name`", e.to_string()),
    }

    Ok(())
}

#[test]
fn test_try_merge_v1() -> Result<(), Box<dyn Error>> {
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
fn test_try_merge_v2() -> Result<(), Box<dyn Error>> {
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
) -> Result<(), Box<dyn Error>> {
    let mut meta = vec![orig];
    for p in patches {
        meta.push(p);
    }
    match Meta::try_from(meta.as_slice()) {
        Err(e) => panic!("patching {name} failed: {e}"),
        Ok(m) => {
            // Convert the Meta object to JSON.
            let output: Result<Value, Box<dyn Error>> = m.try_into();
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
fn test_try_merge_err() -> Result<(), Box<dyn Error>> {
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
            vec![&empty],
            "No spec version found in first meta value",
        ),
        (
            "bad version",
            vec![&bad_version],
            "No spec version found in first meta value",
        ),
        (
            "invalid",
            vec![&invalid],
            "jsonschema validation failed with https://pgxn.org/meta/v2/distribution.schema.json#\n- at '': missing properties 'version'",
        ),
    ] {
        match Meta::try_from(arg.as_slice()) {
            Ok(_) => panic!("patching {name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }

    Ok(())
}

#[test]
fn test_try_merge_partman() -> Result<(), Box<dyn Error>> {
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
    let meta = [&original_meta, &patch];
    match Meta::try_from(&meta[..]) {
        Err(e) => panic!("patching part man failed: {e}"),
        Ok(m) => {
            // Convert the Meta object to JSON.
            let output: Result<Value, Box<dyn Error>> = m.try_into();
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
