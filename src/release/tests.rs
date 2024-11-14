use super::*;
use crate::error::Error;
use chrono::prelude::*;
use serde_json::{json, Value};
use std::{fs::File, io::Cursor, io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use wax::Glob;

fn certs() -> Value {
    json!({
      "certs": {
        "pgxn": {
          "payload": "eyJ1c2VyIjoidGhlb3J5IiwiZGF0ZSI6IjIwMjQtMDktMTNUMTc6MzI6NTVaIiwidXJpIjoiZGlzdC9wYWlyLzAuMS43L3BhaXItMC4xLjcuemlwIiwiZGlnZXN0cyI6eyJzaGE1MTIiOiJiMzUzYjVhODJiM2I1NGU5NWY0YTI4NTllN2EyYmQwNjQ4YWJjYjM1YTdjMzYxMmIxMjZjMmM3NTQzOGZjMmY4ZThlZTFmMTllNjFmMzBmYTU0ZDdiYjY0YmNmMjE3ZWQxMjY0NzIyYjQ5N2JjYjYxM2Y4MmQ3ODc1MTUxNWI2NyJ9fQ",
          "signature": "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q",
        },
        "x_yz": true,
        "x_ab": {"kid": "anna"},
      },
    })
}

fn payload() -> Value {
    json!({
      "user": "theory",
      "date": "2024-07-20T20:34:34Z",
      "uri": "dist/semver/0.40.0/semver-0.40.0.zip",
      "digests": {
        "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"
      }
    })
}

fn release_date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 7, 20, 20, 34, 34).unwrap()
}

#[test]
fn test_corpus() -> Result<(), Error> {
    let certs = certs();
    let payload = get_payload(&certs);
    for (version, patch) in [
        (
            1,
            json!({
              "user": payload.user,
              "date": payload.date,
              "sha1": "0389be689af6992b4da520ec510d147bae411e8b",
            }),
        ),
        (2, certs),
    ] {
        let v_dir = format!("v{version}");
        let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", &v_dir]
            .iter()
            .collect();
        let glob = Glob::new("*.json")?;

        for path in glob.walk(dir) {
            // Load and patch metadata.
            let path = path?.into_path();
            let bn = path.file_name().unwrap().to_str().unwrap();
            let mut meta: Value = serde_json::from_reader(File::open(&path)?)?;
            json_patch::merge(&mut meta, &patch);

            // Test try_from value.
            match Release::try_from(meta.clone()) {
                Err(e) => panic!("{v_dir}/{bn} failed: {e}"),
                Ok(release) => {
                    // Validate that certs were loaded
                    if version == 2 {
                        assert_eq!(
                            meta.get("license").unwrap(),
                            release.license(),
                            "{v_dir}/{bn} license",
                        );
                        let certs: HashMap<String, Value> =
                            serde_json::from_value(meta.get("certs").unwrap().clone()).unwrap();
                        assert_eq!(&certs, release.certs(), "{v_dir}/{bn} release certs");
                        assert_eq!(
                            payload.user,
                            release.release().user(),
                            "{v_dir}/{bn} release user"
                        );

                        // Make sure round-trip produces the same JSON.
                        let output: Result<Value, Error> = release.try_into();
                        match output {
                            Err(e) => panic!("{v_dir}/{bn} failed: {e}"),
                            Ok(val) => {
                                assert_json_diff::assert_json_eq!(&meta, &val);
                            }
                        };
                    }
                }
            }

            // Test try_from string.
            let str = meta.to_string();
            match Release::try_from(&str) {
                Err(e) => panic!("{v_dir}/{bn} failed: {e}"),
                Ok(dist) => {
                    if version == 2 {
                        // Make sure value round-trip produces the same JSON.
                        let output: Result<String, Error> = dist.try_into();
                        match output {
                            Err(e) => panic!("{v_dir}/{bn} failed: {e}"),
                            Ok(val) => {
                                let val: Value = serde_json::from_str(&val)?;
                                assert_json_diff::assert_json_eq!(&meta, &val);
                            }
                        };
                    }
                }
            }

            // Test load path.
            let mut file = NamedTempFile::new()?;
            write!(file, "{str}")?;
            file.flush()?;
            let path = file.path();
            if let Err(e) = Release::load(path) {
                panic!("{v_dir}/{:?} failed: {e}", path.file_name().unwrap());
            }
        }
    }

    Ok(())
}

fn get_payload(meta: &Value) -> ReleasePayload {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let b64 = meta
        .get("certs")
        .unwrap()
        .as_object()
        .unwrap()
        .get("pgxn")
        .unwrap()
        .as_object()
        .unwrap()
        .get("payload")
        .unwrap()
        .as_str()
        .unwrap();
    let json = URL_SAFE_NO_PAD.decode(b64).unwrap();
    serde_json::from_slice(&json).unwrap()
}

#[test]
fn test_bad_corpus() -> Result<(), Error> {
    // Load valid distribution metadata.
    let file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "invalid.json"]
        .iter()
        .collect();
    let bn = file.file_name().unwrap().to_str().unwrap();
    let mut meta: Value = serde_json::from_reader(File::open(&file)?)?;

    // Patch it with release metadata.
    let patch = certs();
    json_patch::merge(&mut meta, &patch);

    // Make sure we catch the validation failure.
    match Release::try_from(meta.clone()) {
        Ok(_) => panic!("Should have failed on {bn} but did not"),
        Err(e) => assert!(
            e.to_string().contains(" missing properties 'version'"),
            "{e}"
        ),
    }

    // Make sure we fail on invalid version.
    match Release::from_version(99, meta.clone()) {
        Ok(_) => panic!("Unexpected success with invalid version"),
        Err(e) => assert_eq!("cannot determine meta-spec version", e.to_string(),),
    }

    // Should fail when no meta-spec.
    meta.as_object_mut().unwrap().remove("meta-spec");
    match Release::try_from(meta.clone()) {
        Ok(_) => panic!("Unexpected success with no meta-spec"),
        Err(e) => assert_eq!("cannot determine meta-spec version", e.to_string()),
    }

    // Should fail on missing certs object.
    let obj = meta.as_object_mut().unwrap();
    obj.insert("meta-spec".to_string(), json!({"version": "2.0.0"}));
    obj.remove("certs");
    match Release::try_from(meta.clone()) {
        Ok(_) => panic!("Unexpected success with no certs property"),
        Err(e) => assert!(e.to_string().contains(" missing properties 'certs'"), "{e}",),
    }

    // Make sure we catch a failure parsing into a Release struct.
    match Release::from_version(2, json!({"invalid": true})) {
        Ok(_) => panic!("Unexpected success with invalid schema"),
        Err(e) => assert_eq!("missing field `certs`", e.to_string()),
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

#[test]
fn test_try_merge_v1() -> Result<(), Error> {
    // Load a v1 META file.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let widget_file = dir.join("v1").join("widget.json");
    let mut contents: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    // Insert release metadata.
    let obj = contents.as_object_mut().unwrap();
    obj.insert("user".to_string(), json!("omar"));
    obj.insert("date".to_string(), json!("2023-07-23T08:54:32.386"));
    obj.insert(
        "sha1".to_string(),
        json!("ca8716f3b0c65ec10207acbe93e09dadbecfbf92"),
    );

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
                "sql": "sql/widget.sql.in",
                "control": "widget.control",
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
    let patch = certs();
    let mut meta = vec![orig, &patch];
    for p in patches {
        meta.push(p);
    }
    match Release::try_from(meta.as_slice()) {
        Err(e) => panic!("patching {name} failed: {e}"),
        Ok(dist) => {
            // Convert the Release object to JSON.
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
            vec![&empty],
            "cannot determine meta-spec version",
        ),
        (
            "bad version",
            vec![&bad_version],
            "cannot determine meta-spec version",
        ),
        ("invalid", vec![&invalid], "missing properties 'version'"),
    ] {
        match Release::try_from(arg.as_slice()) {
            Ok(_) => panic!("patching {name} unexpectedly succeeded"),
            Err(e) => assert!(e.to_string().contains(err), "{name}: {e}"),
        }
    }

    Ok(())
}

#[test]
fn digests() {
    for (name, json) in [
        (
            "sha1",
            json!({"sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a"}),
        ),
        (
            "sha256",
            json!({"sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e"}),
        ),
        (
            "sha512",
            json!({"sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c58"}),
        ),
        (
            "all three",
            json!({
                "sha1": "fe8c013f991b5f537c39fb0c0b04bc955457675a",
                "sha256": "0b68ee2ce5b2c0641c6c429ed2ce17e2ed76ddd58bf9a16e698c5069d60aa34e",
                "sha512": "22e06f682a7fec79f814f06b5dcea0b06133890775ddc624de744cd5d4e8d5fe29863ba5e77c6d3690b610dbcdf7d79a973561fdfbd8454508998446af8f2c58",
            }),
        ),
    ] {
        let dig: Digests = serde_json::from_value(json.clone()).unwrap();
        match json.get("sha1") {
            None => assert!(dig.sha1().is_none(), "{name} url"),
            Some(sha) => assert_eq!(
                sha.as_str().unwrap(),
                hex::encode(dig.sha1().unwrap()),
                "{name} sha1"
            ),
        }
        match json.get("sha256") {
            None => assert!(dig.sha256().is_none(), "{name} url"),
            Some(sha) => assert_eq!(
                sha.as_str().unwrap(),
                hex::encode(dig.sha256().unwrap()),
                "{name} sha256"
            ),
        }
        match json.get("sha512") {
            None => assert!(dig.sha512().is_none(), "{name} url"),
            Some(sha) => assert_eq!(
                sha.as_str().unwrap(),
                hex::encode(dig.sha512().unwrap()),
                "{name} sha512"
            ),
        }
    }
}

#[test]
fn digest_validation() {
    use sha1::Sha1;
    use sha2::{Digest, Sha256, Sha512};

    for (name, data, digests, err) in [
        (
            "sha1 only",
            b"foo".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"foo").into()),
                sha256: None,
                sha512: None,
            },
            None::<&'static str>,
        ),
        (
            "sha1 fail",
            b"foo".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"foobar").into()),
                sha256: None,
                sha512: None,
            },
            Some("SHA-1 digest 0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33 does not match 8843d7f92416211de9ebb963ff4ce28125932878"),
        ),
        (
            "sha256 only",
            b"foobar".to_vec(),
            Digests {
                sha1: None,
                sha256: Some(Sha256::digest(b"foobar").into()),
                sha512: None,
            },
            None,
        ),
        (
            "sha256 fail",
            b"foobar".to_vec(),
            Digests {
                sha1: None,
                sha256: Some(Sha256::digest(b"foo bar lol").into()),
                sha512: None,
            },
            Some("SHA-256 digest c3ab8ff13720e8ad9047dd39466b3c8974e592c2fa383d4a3960714caef0c4f2 does not match bc218a65a558c9b8f313317afa9234220fe2e02b1db629e969ad5f9aa3ec474b"),
        ),
        (
            "sha512 only",
            b"Brat and it's completely different but also still brat".to_vec(),
            Digests {
                sha1: None,
                sha256: None,
                sha512: Some(
                    Sha512::digest(b"Brat and it's completely different but also still brat")
                        .into(),
                ),
            },
            None,
        ),
        (
            "sha512 fail",
            b"Brat and it's completely different but also still brat".to_vec(),
            Digests {
                sha1: None,
                sha256: None,
                sha512: Some(
                    Sha512::digest(b"The Rise and Fall of a Midwest Princess")
                        .into(),
                ),
            },
            Some("SHA-512 digest c2a036d2333446a7e35d10c3c1b2730633ab1e777ce96986a95a6b3c389861b2ce4f0ca36dd17248cafd81dc5da9dec1cc7b0861b3a89145ca6f9b79ea7a2338 does not match 854bb0d8a1c74bb5a908de96bfd3b104e1199f7fca86afea13df508dd03569f10c0e46f001824a380dd2c294fb997ff2a17649a2de5ab94dcdfecc4e738e8b82"),
        ),
        (
            "all three",
            b"So I featuring a. g. cook".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"So I featuring a. g. cook").into()),
                sha256: Some(Sha256::digest(b"So I featuring a. g. cook").into()),
                sha512: Some(Sha512::digest(b"So I featuring a. g. cook").into()),
            },
            None,
        ),
        (
            "all three fail sha512",
            b"So I featuring a. g. cook".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"So I featuring a. g. cook").into()),
                sha256: Some(Sha256::digest(b"So I featuring a. g. cook").into()),
                sha512: Some(Sha512::digest(b"So I featuring a. g. kook").into()),
            },
            Some("SHA-512 digest 6613d453097b5dac724cf1eda4e8b1dda22fb1fef814095a6042b2ff18a581c8d3389c76f07c28b4713dc1b6ab84751d2c04cfd6c09eadbf734010df1fd20981 does not match e836a081212cd0c96f14cbe95ffae2e7ddf57a4deecd0d0810c5f0d5705f32a5d1d93b6ea64a933a9ade762efe2ce049b2250314f61ed72f06c9eca6b6e8a068"),
        ),
        (
            "all three fail sha256",
            b"So I featuring a. g. cook".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"So I featuring a. g. cook").into()),
                sha256: Some(Sha256::digest(b"So I featuring a. g. kook").into()),
                sha512: Some(Sha512::digest(b"So I featuring a. g. cook").into()),
            },
            Some("SHA-256 digest 0a636daa781343cbe78899f4ce6db80884e1f5cba2f321707ed804d8316bfd48 does not match 5327c9bc3812e9f90055db6241c1c2a58f256ec367d449eada565f09a0c51c20"),
        ),
        (
            "all three fail sha1",
            b"So I featuring a. g. cook".to_vec(),
            Digests {
                sha1: Some(Sha1::digest(b"So I featuring a. g. kook").into()),
                sha256: Some(Sha256::digest(b"So I featuring a. g. cook").into()),
                sha512: Some(Sha512::digest(b"So I featuring a. g. cook").into()),
            },
            Some("SHA-1 digest de4031265fa2d0813804105f662c651bbf1c29e6 does not match 50f22ad02e731657438bd6bf1a4437e47312e42d"),
        ),
        (
            "no digests",
            b"foo".to_vec(),
            Digests {
                sha1: None,
                sha256: None,
                sha512: None,
            },
            Some("digests property missing"),
        ),
    ] {
        // std::io::Cursor wraps a vector and provides io::Read and io::Seek.
        let res = digests._validate(Cursor::new(data));
        match err {
            None => assert!(res.is_ok(), "{name}"),
            Some(e) => assert_eq!(e, res.unwrap_err().to_string(), "{name}"),
        }
    }
}

#[test]
fn digest_file_validation() {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "v2"]
        .iter()
        .collect();

    for (file, sha1) in [
        ("minimal.json", "6123adb8f053fe51d11f4fd512c0f1b6953c05b5"),
        ("typical-c.json", "a7fe4df78de863433745ad190a3f2e9a91470197"),
    ] {
        let path = dir.join(file);
        let mut dst = [0u8; 20];
        hex::decode_to_slice(sha1, &mut dst).unwrap();
        let digests = Digests {
            sha1: Some(dst),
            sha256: None,
            sha512: None,
        };

        let res = digests.validate(&path);
        if let Err(e) = digests.validate(&path) {
            panic!("{file} validation failed: {e}")
        }
        assert!(res.is_ok(), "{file}");
    }
}

#[test]
fn release_payload() {
    let payload = payload();
    let date = release_date();
    let sha1 = payload.get("digests").unwrap().get("sha1").unwrap();
    let load: ReleasePayload = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!(payload.get("user").unwrap(), load.user(), "payload name");
    assert_eq!(payload.get("uri").unwrap(), load.uri(), "payload uri");
    assert_eq!(&date, load.date(), "payload date");
    assert_eq!(
        sha1.as_str().unwrap(),
        hex::encode(load.digests().sha1().unwrap()),
        "payload digests",
    )
}

#[test]
fn release() -> Result<(), Error> {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "v2"]
        .iter()
        .collect();
    let glob = Glob::new("*.json")?;

    for path in glob.walk(dir) {
        // Load a v2 META file.
        let path = path?.into_path();
        let name = path.as_path().to_str().unwrap();
        let mut meta: Value = serde_json::from_reader(File::open(&path)?)?;

        // Patch it.
        let patch = certs();
        let payload = get_payload(&patch);
        json_patch::merge(&mut meta, &patch);

        // Load it up.
        match Release::try_from(meta.clone()) {
            Err(e) => panic!("{name} failed: {e}"),
            Ok(rel) => {
                // Should have the certs.
                let certs: HashMap<String, Value> =
                    serde_json::from_value(patch.get("certs").unwrap().clone())?;
                assert_eq!(&certs, rel.certs(), "{name} certs");
                // Should have the release payload.
                assert_eq!(&payload, rel.release(), "{name} release");
                // Required fields.
                assert_eq!(
                    meta.get("name").unwrap().as_str().unwrap(),
                    rel.name(),
                    "{name} name",
                );
                assert_eq!(
                    meta.get("version").unwrap().as_str().unwrap(),
                    rel.version().to_string(),
                    "{name} version",
                );
                assert_eq!(
                    meta.get("abstract").unwrap().as_str().unwrap(),
                    rel.abs_tract().to_string(),
                    "{name} abstract",
                );
                assert_eq!(
                    meta.get("license").unwrap().as_str().unwrap(),
                    rel.license(),
                    "{name} license",
                );

                let val: Spec =
                    serde_json::from_value(meta.get("meta-spec").unwrap().clone()).unwrap();
                assert_eq!(&val, rel.spec(), "{name} spec");

                let val: Vec<Maintainer> =
                    serde_json::from_value(meta.get("maintainers").unwrap().clone()).unwrap();
                assert_eq!(&val, rel.maintainers(), "{name} maintainers");

                let val: Contents =
                    serde_json::from_value(meta.get("contents").unwrap().clone()).unwrap();
                assert_eq!(&val, rel.contents(), "{name} contents");

                // Optional fields.
                match meta.get("description") {
                    None => assert!(rel.description().is_none(), "{name} description"),
                    Some(description) => assert_eq!(
                        description.as_str().unwrap(),
                        rel.description().unwrap(),
                        "{name} description"
                    ),
                }
                match meta.get("producer") {
                    None => assert!(rel.producer().is_none(), "{name} producer"),
                    Some(producer) => assert_eq!(
                        producer.as_str().unwrap(),
                        rel.producer().unwrap(),
                        "{name} producer"
                    ),
                }
                match meta.get("classifications") {
                    None => assert!(rel.classifications().is_none(), "{name} classifications"),
                    Some(val) => {
                        let p: Classifications = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, rel.classifications().unwrap(), "{name} classifications");
                    }
                }
                match meta.get("ignore") {
                    None => assert!(rel.ignore().is_none(), "{name} ignore"),
                    Some(val) => {
                        let p: Vec<String> = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, rel.ignore().unwrap(), "{name} ignore");
                    }
                }
                match meta.get("dependencies") {
                    None => assert!(rel.dependencies().is_none(), "{name} dependencies"),
                    Some(val) => {
                        let p: Dependencies = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, rel.dependencies().unwrap(), "{name} dependencies");
                    }
                }
                match meta.get("resources") {
                    None => assert!(rel.resources().is_none(), "{name} resources"),
                    Some(val) => {
                        let p: Resources = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, rel.resources().unwrap(), "{name} resources");
                    }
                }
                match meta.get("artifacts") {
                    None => assert!(rel.artifacts().is_none(), "{name} artifacts"),
                    Some(val) => {
                        let p: Vec<Artifact> = serde_json::from_value(val.clone()).unwrap();
                        assert_eq!(&p, rel.artifacts().unwrap(), "{name} artifacts");
                    }
                }
                assert_eq!(&exes_from(&meta), rel.custom_props(), "{name} custom_props");
            }
        }
    }

    Ok(())
}

#[test]
fn release_deserialize_err() -> Result<(), Error> {
    // Load a v2 META file.
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus"].iter().collect();
    let widget_file = dir.join("v2").join("minimal.json");
    let meta: Value = serde_json::from_reader(File::open(&widget_file)?)?;

    for (name, patch, err) in [
        ("missing certs", json!({}), "missing field `certs`"),
        (
            "missing pgxn",
            json!({"certs": {}}),
            "invalid or missing pgxn release data",
        ),
        (
            "missing payload",
            json!({"certs": {"pgxn": {}}}),
            "missing or invalid pgxn payload",
        ),
        (
            "invalid base64",
            json!({"certs": {"pgxn": {"payload": "not base64"}}}),
            "Invalid symbol 32, offset 3.",
        ),
        (
            "invalid json",
            json!({"certs": {"pgxn": {"payload": "bm90IGpzb24"}}}),
            "expected ident at line 1 column 2",
        ),
        (
            "invalid payload",
            json!({"certs": {"pgxn": {"payload": "eyJ1c2VyIjogIm5hb21pIn0"}}}),
            "jsonschema validation failed with https://pgxn.org/meta/v2/payload.schema.json#\n- at '': missing properties 'date', 'uri', 'digests'",
        ),
    ] {
        let mut meta = meta.clone();
        json_patch::merge(&mut meta, &patch);

        match Release::deserialize(meta.clone()) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string()),
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
