mod common;
mod v1;
mod v2;

#[test]
fn test_readme_example() {
    use crate::dist::Distribution;
    use serde_json::json;

    // Load the contents of a META.json file into a serde_json::Value.
    let meta = json!({
      "name": "pair",
      "abstract": "A key/value pair data type",
      "version": "0.1.8",
      "maintainer": "theory <theory@pgxn.org>",
      "license": "postgresql",
      "provides": {
        "pair": {
          "file": "sql/pair.sql",
          "version": "0.1.8"
        }
      },
      "meta-spec": { "version": "1.0.0" }
    });

    // Validate and load the META.json contents.
    match Distribution::try_from(meta) {
        Err(e) => panic!("Validation failed: {e}"),
        Ok(dist) => println!("Loaded {} {}", dist.name(), dist.version()),
    };
}
