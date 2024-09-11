use serde_json::Value;

// get_version returns the major version from the value stored in
// `meta-spec.version` in `meta`. Returns None if the field does not exist or
// does not contain a valid version (either 1 or 2).
pub fn get_version(meta: &Value) -> Option<u8> {
    let v = meta.get("meta-spec")?.get("version")?.as_str()?;
    if v.len() < 2 {
        return None;
    }
    match &v[0..2] {
        "1." => Some(1),
        "2." => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_version() {
        for (name, meta, expect) in [
            (
                "1.0.0",
                json!({"meta-spec": { "version": "1.0.0"}}),
                Some(1),
            ),
            (
                "1.0.1",
                json!({"meta-spec": { "version": "1.0.1"}}),
                Some(1),
            ),
            (
                "1.1.0",
                json!({"meta-spec": { "version": "1.1.0"}}),
                Some(1),
            ),
            ("1.", json!({"meta-spec": { "version": "1."}}), Some(1)),
            (
                "2.0.0",
                json!({"meta-spec": { "version": "2.0.0"}}),
                Some(2),
            ),
            (
                "2.0.1",
                json!({"meta-spec": { "version": "2.0.1"}}),
                Some(2),
            ),
            (
                "2.1.0",
                json!({"meta-spec": { "version": "2.1.0"}}),
                Some(2),
            ),
            ("2.", json!({"meta-spec": { "version": "2."}}), Some(2)),
            ("3.", json!({"meta-spec": { "version": "3."}}), None),
            ("3.0.0", json!({"meta-spec": { "version": "3.0.0"}}), None),
            ("9.0.0", json!({"meta-spec": { "version": "9.0.0"}}), None),
            ("too short", json!({"meta-spec": { "version": "1"}}), None),
            ("empty string", json!({"meta-spec": { "version": ""}}), None),
            ("null", json!({"meta-spec": { "version": null}}), None),
            ("bool", json!({"meta-spec": { "version": true}}), None),
            ("number", json!({"meta-spec": { "version": 1}}), None),
            ("object", json!({"meta-spec": { "version": {}}}), None),
            ("array", json!({"meta-spec": { "version": []}}), None),
            ("no version", json!({"meta-spec": {}}), None),
            ("meta-spec array", json!({"meta-spec": [1]}), None),
            ("no meta-spec", json!({}), None),
            ("root array", json!([1]), None),
        ] {
            assert_eq!(expect, get_version(&meta), "{name}")
        }
    }
}
