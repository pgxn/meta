PGXN Meta &emsp; [![license-badge]][license] [![ci-badge]][ci] [![cov-badge]][cov] [![deps-badge]][deps]
=========

**The pgxn_meta crate provides [PGXN Meta Spec] validation**

---

The [PGXN Meta Spec] defines the requirements for the metadata file
(`META.json`) file for [PGXN] source distribution packages. This project
provides Rust a crates for working with spec `META.json` files.

Example
-------

<details>
<summary>Click to show `Cargo.toml`.</summary>

```toml
[dependencies]
serde_json = "1.0"
pgxn_meta = "0.1"
```
</details>

``` rust
use serde_json::json;
use pgxn_meta::*;

func main() {
    // Parse the contents of a META.json file into a serde_json Value
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

    // Validate the META.json contents.
    let mut validator = Validator::new();
    if let Err(e) = validator.validate(&meta) {
        panic!("Validation failed: {e}");
    };
}
```

Contributing
------------

We welcome community contributions to this project. All contributors must
abide by the [PostgresSQL Code of Conduct].

*   Create [Issues] to submit bug reports and feature requests
*   Submit [Pull Requests] to fix issues or add features

License
-------

This project is distributed under the [PostgreSQL License][license].

  [license-badge]: https://img.shields.io/badge/License-PostgreSQL-blue.svg
  [license]: https://opensource.org/licenses/PostgreSQL "⚖️ PostgreSQL License"
  [ci-badge]: https://github.com/pgxn/meta/actions/workflows/test-and-lint.yml/badge.svg
  [ci]: https://github.com/pgxn/meta/actions/workflows/test-and-lint "🧪 Test and Lint"
  [cov-badge]: https://codecov.io/gh/pgxn/meta/graph/badge.svg?token=5DOLLPIHEO
  [cov]: https://codecov.io/gh/pgxn/meta "📊 Code Coverage"
  [deps-badge]: https://deps.rs/repo/github/pgxn/meta/status.svg
  [deps]: https://deps.rs/repo/github/pgxn/meta "📦 Dependency Status"
  [PGXN Meta Spec]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [PGXN]: https://pgxn.org "PGXN: PostgreSQL Extension Network"
  [PostgresSQL Code of Conduct]: https://www.postgresql.org/about/policies/coc/
  [Issues]: https://github.con/pgxn/meta/issues
