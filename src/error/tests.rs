use super::*;
use serde_json::json;

#[test]
fn spdx() {
    let parse_error = spdx::Expression::parse("not a license").unwrap_err();
    let exp = parse_error.reason.to_string();
    let not_exp = parse_error.to_string();
    let err: Error = parse_error.into();
    assert!(matches!(err, Error::License { .. }));
    assert_eq!(exp, err.to_string());
    assert_ne!(not_exp, err.to_string());
}

#[test]
fn unknown_spec() {
    assert_eq!(
        Error::UnknownSpec.to_string(),
        "cannot determine meta-spec version"
    )
}

#[test]
fn unknown_schema_id() {
    assert_eq!(Error::UnknownSchemaId.to_string(), "no $id found in schema")
}

#[test]
fn compile() {
    let mut c = boon::Compiler::new();
    c.add_resource("foo", json!("not a schema")).unwrap();
    let mut s = boon::Schemas::new();
    let compile_err = c.compile("foo", &mut s).unwrap_err();
    let exp = compile_err.to_string();
    let err: Error = compile_err.into();
    assert!(matches!(err, Error::CompileError { .. }));
    assert_eq!(exp, err.to_string(),);
}

#[test]
fn validation() {
    let mut c = boon::Compiler::new();
    c.add_resource("foo", json!({"type": "object"})).unwrap();
    let mut s = boon::Schemas::new();
    let idx = c.compile("foo", &mut s).unwrap();
    let json = json!([]);
    let valid_err = s.validate(&json, idx).unwrap_err();
    let exp = valid_err.to_string();
    let err: Error = valid_err.into();
    assert!(matches!(err, Error::ValidationError { .. }));
    assert_eq!(exp, err.to_string());
}

#[test]
fn serde() {
    let serde_err = serde_json::from_str::<String>("[]").unwrap_err();
    let exp = serde_err.to_string();
    let err: Error = serde_err.into();
    assert!(matches!(err, Error::Serde { .. }));
    assert_eq!(exp, err.to_string());
}

#[test]
fn io() {
    use std::io;
    let io_error = io::Error::new(io::ErrorKind::Other, "oh no!");
    let exp = io_error.to_string();
    let err: Error = io_error.into();
    assert!(matches!(err, Error::Io { .. }));
    assert_eq!(exp, err.to_string());
}

#[test]
fn glob() {
    let build_err = wax::Glob::new("[].json").unwrap_err();
    let exp = build_err.to_string();
    let err: Error = build_err.into();
    assert!(matches!(err, Error::Glob { .. }));
    assert_eq!(exp, err.to_string());

    let glob = wax::Glob::new("*.json").unwrap();
    for path in glob.walk("nonesuch 😇") {
        // Would be nice to just fetch the first, but should always be an
        // error anyway.
        let walk_err = path.unwrap_err();
        let exp = walk_err.to_string();
        let err: Error = walk_err.into();
        assert!(matches!(err, Error::Glob { .. }));
        assert_eq!(exp, err.to_string());
    }
}

#[test]
fn parameter() {
    assert_eq!("invalid hi", Error::Param("invalid hi").to_string())
}

#[test]
fn invalid() {
    for (name, err, exp) in [
        (
            "v1 thing",
            Error::Invalid("thing", 1, json!("hi")),
            "invalid v1 thing value: \"hi\"",
        ),
        (
            "v2 bag array",
            Error::Invalid("bag", 2, json!([])),
            "invalid v2 bag value: []",
        ),
    ] {
        assert_eq!(exp, err.to_string(), "{name}");
    }
}

#[test]
fn missing() {
    assert_eq!(
        "thing property missing",
        Error::Missing("thing").to_string()
    )
}
