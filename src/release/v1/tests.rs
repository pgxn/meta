use super::*;

#[test]
fn test_v1_v2_release() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
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
        // Should have one key.
        assert_eq!(1, v2.as_object().unwrap().keys().len());

        // Make sure we have the pgxn key.
        let pgxn = v2.get("pgxn").unwrap();
        assert_eq!(2, pgxn.as_object().unwrap().keys().len());

        // Extract the payload and make sure the keys are in code point order.
        let pay = pgxn.get("payload").unwrap().as_str().unwrap();
        let pay = URL_SAFE_NO_PAD.decode(pay).unwrap();
        let str = String::from_utf8(pay.clone()).unwrap();
        let mut prev = 0;
        for key in ["date", "digests", "uri", "user"] {
            let idx = str.find(key).unwrap();
            assert!(idx > prev);
            prev = idx;
        }

        // Make sure there is no blank spacing in the payload JSON.
        assert!(!str.contains("\": "));
        assert!(!str.contains("\n"));

        // Decode the payload and make sure it's correct.
        let pay: Value = serde_json::from_slice(&pay).unwrap();
        assert_eq!(input.get("user"), pay.get("user"), "{name} user");
        assert_eq!(input.get("date"), pay.get("date"), "{name} date");
        let uri = Value::String(format!(
            "dist/{0}/{1}/{0}-{1}.zip",
            input.get("name").unwrap().as_str().unwrap(),
            input.get("version").unwrap().as_str().unwrap(),
        ));
        assert_eq!(&uri, pay.get("uri").unwrap(), "{name} uri");

        // Make sure signatures contains a 32-char random string.
        let sig = pgxn.get("signature").unwrap().as_str().unwrap();
        assert_eq!(32, sig.len());
        assert_ne!(&prev_sig, sig, "{name} sig");
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
                format!("{name} property missing"),
                e.to_string(),
                "{name}: {e}"
            ),
        }
    }
}
