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
