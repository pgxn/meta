# Changelog

All notable changes to this project will be documented in this file. It uses the
[Keep a Changelog] format, and this project adheres to [Semantic Versioning].

  [Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
  [Semantic Versioning]: https://semver.org/spec/v2.0.0.html
    "Semantic Versioning 2.0.0"

## [v0.6.1] — Unreleased

### ⚡ Improvements

*   Added a limit to the number of tags in the v2 schema to 32.
*   Added glob format and validation using [wax].
*   Disallow parent directory components (`..`) in globs (already disallowed
    in paths)
*   Disallow current directory components (`.`) in paths and globs except at
    the start of the expression (e.g., `./README`).

  [v0.6.1]: https://github.com/pgxn/meta/compare/v0.6.0...v0.6.1
  [wax]: https://crates.io/crates/wax

## [v0.6.0] — 2025-03-25

### ⬆️ Dependency Updates

*   Upgraded all dependencies.

### ⚡ Improvements

*   Added basic logging for schema validation, digest validation, and JWS
    validation.
*   Extended the v1 Release schema to include the properties added by the [API
    Server Meta API]. However, these properties are not currently available
    via accessors, as the v2 schema does not (yet) define equivalents.
*   Improved validation of v1 schema paths to ensure they always use Unix
    conventions and don't point to files outside a distribution.

### 🏗️ Build Setup

*   Upgraded [Cross] and and the binary build workers.
*   Dropped the Solaris build pending [support for Solaris 11].

  [v0.6.0]: https://github.com/pgxn/meta/compare/v0.5.2...v0.6.0
  [Cross]: https://github.com/cross-rs/cross/
    "“Zero setup” cross compilation and “cross testing” of Rust crates"
  [API Server Meta API]: https://github.com/pgxn/pgxn-api/wiki/meta-api#api-server-structure
  [support for Solaris 11]: https://github.com/cross-rs/cross/issues/1599
    "cross-rs/cross#1599: Support Solaris 11"

## [v0.5.2] — 2025-01-07

### ⬆️ Dependency Updates

*   Upgraded all dependencies, including [boon] v0.6.1, which mitigates
    vulnerabilities.
*   Upgraded all other dependencies

  [v0.5.2]: https://github.com/pgxn/meta/compare/v0.5.1...v0.5.2
  [boon]: https://github.com/santhosh-tekuri/boon

## [v0.5.1] — 2024-11-15

### ⬆️ Dependency Updates

*   Updated all dependencies.

  [v0.5.1]: https://github.com/pgxn/meta/compare/v0.5.0...v0.5.1

## [v0.5.0] — 2024-11-15

### ⚡ Improvements

*   Added the [error module], which defines all the errors returned by
    pgxn_meta.
*   Changed the errors returned by all the APIs from boxed errors [error
    module] errors.
*   Added `release.Digests.validate` method to validate a file against one or
    more digests.

### 📔 Notes

*   Removed the `valid::ValidationError` enum.
*   Changed the errors returned from the [valid module] from boxed [boon]
    errors with lifetimes to [error module] errors with no lifetimes,
    eliminating the need for lifetimes on the Validator struct and methods.

### 📚 Documentation

*   Fixed the repository link in `Cargo.toml`.

  [v0.5.0]: https://github.com/pgxn/meta/compare/v0.4.0...v0.5.0
  [error module]: https://docs.rs/pgxn_meta/0.5.0/pgxn_meta/error/
  [valid module]: https://docs.rs/pgxn_meta/0.5.0/pgxn_meta/valid/
  [boon]: https://github.com/santhosh-tekuri/boon

## [v0.4.0] — 2024-10-08

The theme of this release is *JSON Web Signatures.*

### ⚡ Improvements

*   Following [RFC 5], added v2 JSON Schemas for the `certs` property and its
    child `pgxn` property, which contains an [RFC 7515] JSON Web Signature
    (JWS) [JSON Serialization] object in either the general or flattened
    syntaxes.
*   Revamped the [release module] to support the updated release signing spec
    defined in [RFC 5]
*   Added the `validate_payload` method to `valid::Validator`, which that the
    `release::Release` deserialization implementation uses to validate the JWS
    payload. Required because the payload is Base 64 URL-encoded and therefore
    cannot be parsed into a struct on the first pass.

### 📔 Notes

*   The [release] interface has changed with the new data structures. The
    [JWS-JS] data added in v0.3.0 has been replaced with [RFC 7515]-standard
    [JSON Serialization].

### ⬆️ Dependency Updates

*   Upgraded to [json-patch] v3.0 and updated all other dependencies.

  [v0.4.0]: https://github.com/pgxn/meta/compare/v0.3.0...v0.4.0
  [release module]: https://docs.rs/pgxn_meta/0.4.0/pgxn_meta/release/
  [RFC 5]: https://github.com/pgxn/rfcs/pull/5
    "pgxn/rfs#5 Add RFC for JWS-signing PGXN releases"
  [RFC 7515]: https://datatracker.ietf.org/doc/html/rfc7515 "RFC 7515 JSON Web Signature"
  [JSON Serialization]: https://datatracker.ietf.org/doc/html/rfc7515#section-7.2
    "RFC 7515 JWS — JWS JSON Serialization"

## [v0.3.0] — 2024-09-23

The theme of this release is *Release metadata.*

### ⚡ Improvements

*   Designed experimental distribution [metadata schema] to be appended to v2
    `META.json` by PGXN upon release. The object is in [JWS-JS] format, and
    intended to sign the release user, date, URI, and one or more SHA digests
    for the distribution zip file. The format is subject to change pending
    expert review and approval of the [JWS-signing RFC].
*   Created release JSON Schemas for [v1] and [v2] release validation. PGXN
    Manager adds the v1 metadata to the distribution-supplied `META.json` so
    that clients can validate downloads. In the future it will generate the v2
    JWS-signed schema.
*   Added the [release module], which extends the [dist module] to load loads
    v1 and v2 spec files into read-only data structures, converts v1 metadata
    to v2, and merges multiple files.

### 📔 Notes

*   Renamed the meta module to the [dist module], since it handles
    *distribution* metadata, and therefore better compliments the new [release
    module], which handles *release* metadata.
*   Removed the SHA-256 hash from the [v2 artifacts schema], leaving only
    SHA-512.
*   Replaced the `TryFrom<PathBuf>` trait in the [dist module] with a `load`
    function. This is because one does not convert a file path into a struct,
    but loads it into a struct. It also allows the argument to be of type
    `AsRef<Path>`, which supports `Path`, `PathBuf`, or `String` arguments.
*   The v1-v2 conversion in the [release module] does not sign the release
    payload, as we are not doing any key signing, yet. For now it generates
    random strings to satisfy JSON Schema validation.

  [v0.3.0]: https://github.com/pgxn/meta/compare/v0.2.0...v0.3.0
  [metadata schema]: https://github.com/pgxn/meta/blob/v0.3.0/schema/v2/pgxn-jws.schema.json
  [dist module]: https://docs.rs/pgxn_meta/0.3.0/pgxn_meta/dist/
  [release module]: https://docs.rs/pgxn_meta/0.3.0/pgxn_meta/release/
  [JWS-JS]: https://datatracker.ietf.org/doc/html/draft-jones-json-web-signature-json-serialization-01
  [JWS-signing RFC]: https://github.com/pgxn/rfcs/pull/5
  [v1]: https://github.com/pgxn/meta/blob/v0.3.0/schema/v1/release.schema.json
  [v2]: https://github.com/pgxn/meta/blob/v0.3.0/schema/v2/release.schema.json
  [v2 artifacts schema]: https://github.com/pgxn/meta/blob/v0.3.0/schema/v2/artifacts.schema.json

## [v0.2.0] — 2024-09-12

The theme of this release is *Data structures and APIs.*

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
*   Updated the README example to use the [meta module] to load an object.

  [v0.2.0]: https://github.com/pgxn/meta/compare/v0.1.0...v0.2.0
  [meta module]: https://docs.rs/pgxn_meta/0.2.0/pgxn_meta/meta/
  [valid module]: https://docs.rs/pgxn_meta/0.2.0/pgxn_meta/valid/
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
