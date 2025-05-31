use super::*;
use crate::error::Error;
use serde_json::{json, Value};
use std::{fs::File, path::PathBuf};
use wax::Glob;

#[test]
fn test_corpus() -> Result<(), Error> {
    let mut validator = Validator::default();

    for (version, release_patch) in [
        (
            1,
            json!({
              "user": "theory",
              "date": "2019-09-23T17:16:45Z",
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b",
            }),
        ),
        (
            2,
            json!({"certs": {
              "pgxn": {
                "payload": "abcdefghijkl",
                "signature": "abcdefghijklmnopqrstuvwxyz012345",
              }
            }}),
        ),
    ] {
        let v_dir = format!("v{version}");
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", &v_dir]
            .iter()
            .collect();
        let glob = Glob::new("*.json")?;

        for path in glob.walk(dir) {
            // Load metadata.
            let path = path?.into_path();
            let bn = path.file_name().unwrap().to_str().unwrap();
            let mut meta: Value = serde_json::from_reader(File::open(&path)?)?;

            // Should validate.
            match validator.validate(&meta) {
                Err(e) => panic!("{v_dir}/{bn} validate failed: {e}"),
                Ok(v) => assert_eq!(version, v, "{v_dir}/{bn} validate version"),
            };

            // Patch with release data and validate as release.
            json_patch::merge(&mut meta, &release_patch);
            match validator.validate_release(&meta) {
                Err(e) => panic!("{v_dir}/{bn} validate_release failed: {e}"),
                Ok(v) => assert_eq!(version, v, "{v_dir}/{bn} validate_release version"),
            };
        }
    }

    Ok(())
}

#[test]
fn test_unknown_versions() -> Result<(), Error> {
    let mut validator = Validator::new();

    for (name, json) in [
        ("no meta spec", json!({})),
        ("meta spec array", json!({"meta-spec": []})),
        ("no meta version", json!({"meta-spec": {}})),
        ("meta version bool", json!({"meta-spec": true})),
        ("bad meta version", json!({"meta-spec": {"version": "0.0"}})),
    ] {
        match validator.validate(&json) {
            Err(e) => assert_eq!(
                "cannot determine meta-spec version",
                e.to_string(),
                "{name} validate"
            ),
            Ok(_) => panic!("{name} validate unexpectedly succeeded"),
        }
        match validator.validate_release(&json) {
            Err(e) => assert_eq!(
                "cannot determine meta-spec version",
                e.to_string(),
                "{name} validate_release"
            ),
            Ok(_) => panic!("{name} validate validate_release succeeded"),
        }
    }

    Ok(())
}

fn load_minimal() -> Result<(Value, Value), Error> {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let file = dir.join("v1").join("howto.json");
    let v1: Value = serde_json::from_reader(File::open(file)?)?;
    let file = dir.join("v2").join("minimal.json");
    let v2: Value = serde_json::from_reader(File::open(file)?)?;
    Ok((v1, v2))
}

#[test]
fn test_invalid_distribution() -> Result<(), Error> {
    let mut validator = Validator::new();
    let (v1, v2) = load_minimal()?;

    for (name, meta, patch, err) in [
        (
            "v1 no name",
            &v1,
            json!({"name": null}),
            "missing properties 'name'",
        ),
        (
            "v1 no version",
            &v1,
            json!({"version": null}),
            "missing properties 'version'",
        ),
        (
            "v1 invalid license",
            &v1,
            json!({"license": "lol no"}),
            "'/license': oneOf failed, none matched",
        ),
        (
            "v1 missing provides version",
            &v1,
            json!({"provides": {"pair": {"version": null}}}),
            "missing properties 'version'",
        ),
        (
            "v2 no name",
            &v2,
            json!({"name": null}),
            "missing properties 'name'",
        ),
        (
            "v2 no version",
            &v2,
            json!({"version": null}),
            "missing properties 'version'",
        ),
        (
            "v2 invalid license",
            &v2,
            json!({"license": "lol no"}),
            "'/license': 'lol no' is not valid license: unknown term",
        ),
        (
            "v1 missing control",
            &v1,
            json!({"contents": {"extensions": {"pair": {"control": null}}}}),
            "'/contents': false schema",
        ),
    ] {
        let mut meta = meta.clone();
        json_patch::merge(&mut meta, &patch);
        match validator.validate(&meta) {
            Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
            Ok(_) => panic!("{name} validate unexpectedly succeeded"),
        };

        match validator.validate_release(&meta) {
            Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
            Ok(_) => panic!("{name} validate_release unexpectedly succeeded"),
        };
    }

    Ok(())
}

#[test]
fn test_invalid_release() -> Result<(), Error> {
    let mut validator = Validator::new();
    let (v1, v2) = load_minimal()?;
    for (name, meta, patch, err) in [
        (
            "v1 no sha",
            &v1,
            json!({"user": "xxx", "date": "2019-09-23T17:16:45Z"}),
            "missing properties 'sha1'",
        ),
        (
            "v1 no user",
            &v1,
            json!({"sha1": "0389be689af6992b4da520ec510d147bae411e8b", "date": "2019-09-23T17:16:45Z"}),
            "missing properties 'user'",
        ),
        (
            "v1 no date",
            &v1,
            json!({"user": "xxx", "sha1": "0389be689af6992b4da520ec510d147bae411e8b"}),
            "missing properties 'date'",
        ),
        ("v2 no certs", &v2, json!({}), "missing properties 'certs'"),
        (
            "v2 no pgxn",
            &v2,
            json!({"certs": {}}),
            "'/certs': missing properties 'pgxn'",
        ),
        (
            "v2 no payload",
            &v2,
            json!({"certs": {"pgxn": {"signature": "abcdefghijklmnopqrstuvwxyz012345"}}}),
            "'/certs/pgxn': missing properties 'payload'",
        ),
        (
            "v2 no signature",
            &v2,
            json!({"certs": {"pgxn": {"payload": "abcdefghijkl"}}}),
            "'/certs/pgxn': missing properties 'signature'",
        ),
        (
            "v2 no signatures",
            &v2,
            json!({"certs": {"pgxn": {"payload": "abcdefghijkl"}}}),
            "'/certs/pgxn': missing properties 'signatures'",
        ),
    ] {
        let mut meta = meta.clone();
        json_patch::merge(&mut meta, &patch);

        match validator.validate_release(&meta) {
            Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
            Ok(_) => panic!("{name} validate_release unexpectedly succeeded"),
        };
    }

    Ok(())
}

#[test]
fn test_payload() -> Result<(), Error> {
    let mut validator = Validator::new();
    for (name, payload) in [
        (
            "sha1",
            json!({
              "user": "theory",
              "date": "2024-07-20T20:34:34Z",
              "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
              "digests": {
                "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
              }
            }),
        ),
        (
            "multiple digests",
            json!({
            "user": "theory",
            "date": "2024-09-13T17:32:55Z",
            "uri": "dist/pair/0.1.7/pair-0.1.7.zip",
            "digests": {
                "sha256": "257b71aa57a28d62ddbb301333b3521ea3dc56f17551fa0e4516b03998abb089",
                "sha512": "b353b5a82b3b54e95f4a2859e7a2bd0648abcb35a7c3612b126c2c75438fc2f8e8ee1f19e61f30fa54d7bb64bcf217ed1264722b497bcb613f82d78751515b67"
            }
            }),
        ),
    ] {
        if let Err(e) = validator.validate_payload(&payload) {
            panic!("{name} validate failed: {e}");
        }
    }

    let pay = json!({
      "user": "theory",
      "date": "2024-07-20T20:34:34Z",
      "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
      "digests": {
        "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
      }
    });
    for (name, patch, err) in [
        (
            "no user",
            json!({"user": null}),
            "'': missing properties 'user'",
        ),
        (
            "no date",
            json!({"date": null}),
            "'': missing properties 'date'",
        ),
        (
            "no uri",
            json!({"uri": null}),
            "'': missing properties 'uri'",
        ),
        (
            "no digests",
            json!({"digests": null}),
            "'': missing properties 'digests'",
        ),
        (
            "empty digests",
            json!({"digests": {"sha1": null}}),
            "'/digests': minimum 1 properties required, but got 0 properties",
        ),
    ] {
        let mut pay = pay.clone();
        json_patch::merge(&mut pay, &patch);

        match validator.validate_payload(&pay) {
            Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
            Ok(_) => panic!("{name} validate_payload unexpectedly succeeded"),
        };
    }

    Ok(())
}
