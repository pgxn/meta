# PGXN Meta

[![license-badge]][license] [![crates-badge]][crates] [![docs-badge]][docs] [![ci-badge]][ci] [![cov-badge]][cov] [![deps-badge]][deps]


**The pgxn_meta crate provides [PGXN Meta Spec] validation**

---

The [PGXN Meta Spec] defines the requirements for the metadata file
(`META.json`) file for [PGXN] source distribution packages. This project
provides Rust a crates for working with spec `META.json` files.

Crate Usage
-----------

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

See the [`pgxn_meta` docs on docs.rs] for complete details.

Installation
------------

There are several ways to install `pgxn_meta`.

### `ubi`

Install the [universal binary installer (ubi)][ubi] and use it to install
`pgxn_meta` and many other tools.

``` sh
ubi --project pgxn/meta --in ~/bin
```

### Binary Releases

Grab the appropriate binary [release], untar or unzip it, and put the
`pgxn_meta` executable somewhere in your path.

### Cargo

Compile and install `pgxn_meta` via `cargo` by running:

``` sh
cargo install pgxn_meta
```

See the [cargo docs] to learn where the binary will be installed.

Usage
-----

Simply execute `pgxn_meta` to validate the PGXN `META.json` file in the
current directory:

``` sh
pgxn_meta
```

If the file has a different name, simply pass it:

``` sh
pgxn_meta widget.json
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
  [crates-badge]: https://img.shields.io/crates/v/pgxn_meta.svg
  [crates]: https://crates.io/crates/pgxn_meta
  [docs-badge]: https://docs.rs/pgxn_meta/badge.svg
  [docs]: https://docs.rs/pgxn_meta
  [ci-badge]: https://github.com/pgxn/meta/actions/workflows/test-and-lint.yml/badge.svg
  [ci]: https://github.com/pgxn/meta/actions/workflows/test-and-lint "🧪 Test and Lint"
  [cov-badge]: https://codecov.io/gh/pgxn/meta/graph/badge.svg?token=5DOLLPIHEO
  [cov]: https://codecov.io/gh/pgxn/meta "📊 Code Coverage"
  [deps-badge]: https://deps.rs/repo/github/pgxn/meta/status.svg
  [deps]: https://deps.rs/repo/github/pgxn/meta "📦 Dependency Status"
  [PGXN Meta Spec]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [PGXN]: https://pgxn.org "PGXN: PostgreSQL Extension Network"
  [`pgxn_meta` docs on docs.rs]: https://docs.rs/ubi/latest/pgxn_meta/
  [ubi]: https://github.com/houseabsolute/ubi
  [release]: https://github.com/pgxn/meta/releases
  [cargo docs]: https://doc.rust-lang.org/cargo/commands/cargo-install.html
  [PostgresSQL Code of Conduct]: https://www.postgresql.org/about/policies/coc/
  [Issues]: https://github.com/pgxn/meta/issues
  [Pull Requests]: https://github.com/pgxn/meta/pulls
