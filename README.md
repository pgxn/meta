# PGXN Distribution Metadata

[![license-badge]][license] [![crates-badge]][crates] [![release-badge]][release] [![ci-badge]][ci] [![cov-badge]][cov] [![docs-badge]][docs] [![deps-badge]][deps]


**The pgxn_meta crate provides PGXN Meta [v1] and [v2] validation and management**

---

The PGXN Meta [v1] and [v2] specs define the requirements for the metadata
file (`META.json`) file for [PGXN] source distribution packages. This project
provides Rust a crates for working with spec `META.json` files.

Crate Usage
-----------

<details>
<summary>Click to show `Cargo.toml`.</summary>

```toml
[dependencies]
serde_json = "1.0"
pgxn_meta = "0.2"
```
</details>

``` rust
use serde_json::json;
use pgxn_meta::meta::Distribution;

func main() {
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
```

See the [`pgxn_meta` docs on docs.rs] for complete details.

Installation
------------

There are several ways to install `pgxn_meta`.

### `ubi`

Install the [universal binary installer (ubi)][ubi] and use it to install
`pgxn_meta` and many other tools.

``` sh
ubi --project pgxn/meta --exe pgxn_meta --in ~/bin
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

  [license-badge]: https://img.shields.io/badge/License-PostgreSQL-blue.svg "⚖️ PostgreSQL License"
  [license]: https://opensource.org/licenses/PostgreSQL "⚖️ PostgreSQL License"
  [crates-badge]: https://img.shields.io/crates/v/pgxn_meta.svg "📦 Crate"
  [crates]: https://crates.io/crates/pgxn_meta "📦 Crate"
  [docs-badge]: https://docs.rs/pgxn_meta/badge.svg "📚 Docs Status"
  [docs]: https://docs.rs/pgxn_meta "📚 Docs Status"
  [ci-badge]: https://github.com/pgxn/meta/actions/workflows/test-and-lint.yml/badge.svg "🧪 Test and Lint"
  [ci]: https://github.com/pgxn/meta/actions/workflows/test-and-lint "🧪 Test and Lint"
  [cov-badge]: https://codecov.io/gh/pgxn/meta/graph/badge.svg?token=5DOLLPIHEO "📊 Code Coverage"
  [cov]: https://codecov.io/gh/pgxn/meta "📊 Code Coverage"
  [deps-badge]: https://deps.rs/repo/github/pgxn/meta/status.svg "⬆️ Dependency Status"
  [deps]: https://deps.rs/repo/github/pgxn/meta "⬆️ Dependency Status"
  [release-badge]: https://img.shields.io/github/release/pgxn/meta.svg  "🚀 Latest Release"
  [release]: https://github.com/pgxn/meta/releases/latest "🚀 Latest Release"
  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3
  [PGXN]: https://pgxn.org "PGXN: PostgreSQL Extension Network"
  [`pgxn_meta` docs on docs.rs]: https://docs.rs/ubi/latest/pgxn_meta/
  [ubi]: https://github.com/houseabsolute/ubi
  [release]: https://github.com/pgxn/meta/releases
  [cargo docs]: https://doc.rust-lang.org/cargo/commands/cargo-install.html
  [PostgresSQL Code of Conduct]: https://www.postgresql.org/about/policies/coc/
  [Issues]: https://github.com/pgxn/meta/issues
  [Pull Requests]: https://github.com/pgxn/meta/pulls
