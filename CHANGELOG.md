# Changelog

All notable changes to this project will be documented in this file. It uses the
[Keep a Changelog] format, and this project adheres to [Semantic Versioning].

  [Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
  [Semantic Versioning]: https://semver.org/spec/v2.0.0.html
    "Semantic Versioning 2.0.0"

## [v0.2.0] — Date TBD

### ⚡ Improvements

*   Added the [meta module], which loads v1 and v2 spec files into read-only
    data structures, converts v1 metadata to v2, and merges multiple files.

### 🪲 Bug Fixes

*   Changed the v1 validator to allow `http` as well as `https` in the
    `meta-spec` object's `url` field, as a lot of older `META.json` files use
    it.

### 📔 Notes

*   Moved the validation functionality to the [valid module].

### 📚 Documentation

*   Updated the `v2` link in all docs to point to the [pull request], since it
    hasn't been merged and published yet.

  [v0.2.0]: https://github.com/pgxn/meta/compare/v0.1.0...v0.2.0
  [meta module]: https://docs.rs/pgxn_meta/meta/
  [valid module]: https://docs.rs/pgxn_meta/meta/
  [pull request]: https://github.com/pgxn/rfcs/pull/3 "pgxn/rfcs#3 Meta Spec v2"

## [v0.1.0] — 2024-08-08

The theme of this release is *Cross Compilation.*

### ⚡ Improvements

*   First release, everything is new!
*   JSON Schema for PGXN Meta Spec v1 and v2
*   JSON Schema validation using [boon]
*   Comprehensive Testing
*   `pgxn_meta` binary and crate

### 🏗️ Build Setup

*   Built with Rust
*   Use [cross] and [actions-rust-cross] to cross-compile and release binaries
    for multiple OSes
*   Install from [crates.io] or [GitHub]

### 📚 Documentation

*   Build and install docs in the [README]

  [v0.1.0]: https://github.com/pgxn/meta/compare/4c207a6...v0.1.0
  [boon]: https://github.com/santhosh-tekuri/boon
  [cross]: https://github.com/cross-rs/cross
  [actions-rust-cross]: https://github.com/houseabsolute/actions-rust-cross
  [crates.io]: https://crates.io/crates/pgxn_meta
  [GitHub]: https://github.com/pgxn/meta/releases
  [README]: https://github.com/pgxn/meta/blob/v0.1.0/README.md
