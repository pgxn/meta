use super::*;

#[test]
fn test_v1_v2_release() {
    let mut prev_head = "x".repeat(16);
    let mut prev_sig = "x".repeat(32);

    for (name, input) in [
        (
            "basics",
            json!({
              "name": "pair",
              "version": "1.2.3",
              "user": "xxx",
              "date": "2024-09-18T15:38:15Z",
              "sha1": "d833511c7ebb9c1875426ca8a93edcacd0787c46",
            }),
        ),
        (
            "another",
            json!({
              "name": "semver",
              "version": "3.4.5",
              "user": "yyy",
              "date": "2022-04-12T45:42:21.392Z",
              "sha1": "3e59eff6779d2444d7e120436f675ff2868a0b39",
            }),
        ),
    ] {
        let v2 = v1_to_v2_release(&input).unwrap();
        // Should have three keys.
        assert_eq!(3, v2.as_object().unwrap().keys().len());

        // Make sure the payload is correct.
        let pay = v2.get("payload").unwrap();
        assert_eq!(input.get("user"), pay.get("user"), "{name} user");
        assert_eq!(input.get("date"), pay.get("date"), "{name} date");
        let uri = Value::String(format!(
            "dist/{0}/{1}/{0}-{1}.zip",
            input.get("name").unwrap().as_str().unwrap(),
            input.get("version").unwrap().as_str().unwrap(),
        ));
        assert_eq!(&uri, pay.get("uri").unwrap(), "{name} uri");

        // Make sure headers contains 1 16-char random string.
        let heads = v2.get("headers").unwrap().as_array().unwrap();
        assert_eq!(1, heads.len());
        let head = heads.first().unwrap().as_str().unwrap();
        assert_eq!(16, head.len());
        assert!(head.starts_with("eyJ"));
        assert_ne!(&prev_head, head);
        prev_head = head.to_string();

        // Make sure signatures contains 1 32-char random string.
        let sigs = v2.get("signatures").unwrap().as_array().unwrap();
        assert_eq!(1, sigs.len());
        let sig = sigs.first().unwrap().as_str().unwrap();
        assert_eq!(32, sig.len());
        assert_ne!(&prev_sig, sig);
        prev_sig = sig.to_string();
    }
}

#[test]
fn test_v1_v2_release_err() {
    for (name, input) in [
        ("user", json!({})),
        ("date", json!({"user": "xxx"})),
        ("sha1", json!({"user": "xxx", "date": "today"})),
        (
            "name",
            json!({"user": "xxx", "date": "today", "sha1": "123"}),
        ),
        (
            "version",
            json!({"user": "xxx", "date": "today", "sha1": "123", "name": "pair"}),
        ),
    ] {
        match v1_to_v2_release(&input) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(
                format!("missing release property \"{name}\""),
                e.to_string(),
                "{name}: {e}"
            ),
        }
    }
}
