/*!
PGXN Metadata management.

This module provides interfaces to load, validate, and manipulate PGXN Meta
Spec `META.json` files. It supports both the [v1] and [v2] specs.

  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3

*/
use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use crate::util;
use relative_path::RelativePathBuf;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod v1;
mod v2;

/// Represents the `meta-spec` object in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Spec {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// Maintainer represents an object in the list of `maintainers` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Maintainer {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// Describes an extension in under `extensions` in [`Contents`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Extension {
    control: RelativePathBuf,
    #[serde(rename = "abstract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    abs_tract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tle: Option<bool>,
    sql: RelativePathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<RelativePathBuf>,
}

/// Defines a type of module in [`Module`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum ModuleType {
    #[serde(rename = "extension")]
    Extension,
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "bgw")]
    Bgw,
}

/// Defines the values for the `preload` value in [`Module`]s.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Preload {
    #[serde(rename = "server")]
    Server,
    #[serde(rename = "session")]
    Session,
}

/// Represents a loadable module under `modules` in [`Contents`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Module {
    #[serde(rename = "type")]
    kind: ModuleType,
    #[serde(rename = "abstract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    abs_tract: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preload: Option<Preload>,
    lib: RelativePathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<RelativePathBuf>,
}

/// Represents an app under `apps` in [`Contents`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct App {
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<String>,
    #[serde(rename = "abstract")]
    #[serde(skip_serializing_if = "Option::is_none")]
    abs_tract: Option<String>,
    bin: RelativePathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lib: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    man: Option<RelativePathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<RelativePathBuf>,
}

/// Represents the contents of a distribution, under `contents` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Contents {
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<HashMap<String, Extension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modules: Option<HashMap<String, Module>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apps: Option<HashMap<String, App>>,
}

/// Represents the classifications of a distribution, under `classifications`
/// in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Classifications {
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Vec<String>>,
}

/// Represents Postgres requirements under `postgres` in [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Postgres {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<Vec<String>>,
}

/// Represents the name of a build pipeline under `pipeline` in
/// [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Pipeline {
    /// PGXS
    #[serde(rename = "pgxs")]
    Pgxs,
    #[serde(rename = "meson")]
    /// Meson
    Meson,
    #[serde(rename = "pgrx")]
    /// pgrx
    Pgrx,
    /// Autoconf
    #[serde(rename = "autoconf")]
    Autoconf,
    /// cmake
    #[serde(rename = "cmake")]
    Cmake,
}

/// Defines a version range for [`Phase`] dependencies.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum VersionRange {
    /// Represents `0` as a shorthand for "no specific version".
    Integer(u8),
    /// Represents a string defining a version range.
    String(String),
}

/// Defines the relationships for a build phase in [`Packages`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Phase {
    #[serde(skip_serializing_if = "Option::is_none")]
    requires: Option<HashMap<String, VersionRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recommends: Option<HashMap<String, VersionRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggests: Option<HashMap<String, VersionRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflicts: Option<HashMap<String, VersionRange>>,
}

/// Defines package dependencies for build phases under `packages` in
/// [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Packages {
    #[serde(skip_serializing_if = "Option::is_none")]
    configure: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    test: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run: Option<Phase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    develop: Option<Phase>,
}

/// Defines dependency variations under `variations`in  [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Variations {
    #[serde(rename = "where")]
    wheres: Box<Dependencies>,
    dependencies: Box<Dependencies>,
}

/// Defines the distribution dependencies under `dependencies` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Dependencies {
    #[serde(skip_serializing_if = "Option::is_none")]
    platforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    postgres: Option<Postgres>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pipeline: Option<Pipeline>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packages: Option<Packages>,
    #[serde(skip_serializing_if = "Option::is_none")]
    variations: Option<Vec<Variations>>,
}

/// Defines the badges under `badges` in [`Resources`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Badge {
    src: String,
    alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// Defines the resources under `resources` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Resources {
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    support: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    badges: Option<Vec<Badge>>,
}

/// Defines the artifacts in the array under `artifacts` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Artifact {
    url: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha512: Option<String>,
}

/**
Represents a complete PGXN Meta definition.
*/
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Meta {
    name: String,
    version: Version,
    #[serde(rename = "abstract")]
    abs_tract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    producer: Option<String>,
    license: String, // use spdx::Expression.
    #[serde(rename = "meta-spec")]
    spec: Spec,
    maintainers: Vec<Maintainer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classifications: Option<Classifications>,
    contents: Contents,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Dependencies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<Resources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifacts: Option<Vec<Artifact>>,
}

impl Meta {
    /// Deserializes `meta`, which contains PGXN `version` metadata, into a
    /// [`Meta`].
    fn from_version(version: u8, meta: Value) -> Result<Self, Box<dyn Error>> {
        match version {
            1 => v1::from_value(meta),
            2 => v2::from_value(meta),
            _ => Err(Box::from(format!("Unknown meta version {version}"))),
        }
    }

    /// Returns the license string.
    pub fn license(&self) -> &str {
        self.license.as_str()
    }
}

impl TryFrom<Value> for Meta {
    type Error = Box<dyn Error>;
    /// Converts the PGXN `META.json` data from `meta` into a [`Meta`].
    /// Returns an error if `meta` is invalid.
    ///
    /// # Example
    ///
    /// ``` rust
    /// # use std::error::Error;
    /// use serde_json::json;
    /// use pgxn_meta::meta::*;
    ///
    /// let meta_json = json!({
    ///   "name": "pair",
    ///   "abstract": "A key/value pair data type",
    ///   "version": "0.1.8",
    ///   "maintainers": [
    ///     { "name": "Barrack Obama",  "email": "pogus@example.com" }
    ///   ],
    ///   "license": "PostgreSQL",
    ///   "contents": {
    ///     "extensions": {
    ///       "pair": {
    ///         "sql": "sql/pair.sql",
    ///         "control": "pair.control"
    ///       }
    ///     }
    ///   },
    ///   "meta-spec": { "version": "2.0.0" }
    /// });
    ///
    ///
    /// let meta = Meta::try_from(meta_json);
    /// assert!(meta.is_ok());
    /// ```
    fn try_from(meta: Value) -> Result<Self, Self::Error> {
        // Make sure it's valid.
        let mut validator = crate::valid::Validator::new();
        let version = match validator.validate(&meta) {
            Err(e) => return Err(Box::from(e.to_string())),
            Ok(v) => v,
        };
        Meta::from_version(version, meta)
    }
}

impl TryFrom<&[&Value]> for Meta {
    type Error = Box<dyn Error>;
    /// Merge multiple PGXN `META.json` data from `meta` into a [`Meta`].
    /// Returns an error if `meta` is invalid.
    ///
    /// The first value in `meta` should be the primary metadata, generally
    /// included in a distribution. Subsequent values will be merged into that
    /// first value via the [RFC 7396] merge pattern.
    ///
    /// # Example
    ///
    /// ``` rust
    /// # use std::error::Error;
    /// use serde_json::json;
    /// use pgxn_meta::meta::*;
    ///
    /// let meta_json = json!({
    ///   "name": "pair",
    ///   "abstract": "A key/value pair data type",
    ///   "version": "0.1.8",
    ///   "maintainers": [
    ///     { "name": "Barrack Obama",  "email": "pogus@example.com" }
    ///   ],
    ///   "license": "PostgreSQL",
    ///   "contents": {
    ///     "extensions": {
    ///       "pair": {
    ///         "sql": "sql/pair.sql",
    ///         "control": "pair.control"
    ///       }
    ///     }
    ///   },
    ///   "meta-spec": { "version": "2.0.0" }
    /// });
    ///
    /// let patch = json!({"license": "MIT"});
    /// let all_meta = [&meta_json, &patch];
    ///
    /// let meta = Meta::try_from(&all_meta[..]);
    /// assert!(meta.is_ok());
    /// assert_eq!("MIT", meta.unwrap().license());
    /// ```
    ///
    /// [RFC 7396]: https:///www.rfc-editor.org/rfc/rfc7396.html
    fn try_from(meta: &[&Value]) -> Result<Self, Self::Error> {
        if meta.is_empty() {
            return Err(Box::from("meta contains no values"));
        }

        // Find the version of the first doc.
        let version =
            util::get_version(meta[0]).ok_or("No spec version found in first meta value")?;

        // Convert the first doc to v2 if necessary.
        let mut v2 = match version {
            1 => v1::to_v2(meta[0])?,
            2 => meta[0].clone(),
            _ => unreachable!(),
        };

        // Merge them.
        for patch in meta[1..].iter() {
            json_patch::merge(&mut v2, patch)
        }

        // Validate the patched doc and return.
        let mut validator = crate::valid::Validator::new();
        validator.validate(&v2).map_err(|e| e.to_string())?;
        Meta::from_version(2, v2)
    }
}

impl TryFrom<Meta> for Value {
    type Error = Box<dyn Error>;
    /// Converts `meta` into a [serde_json::Value].
    ///
    /// # Example
    ///
    /// ``` rust
    /// # use std::error::Error;
    /// use serde_json::{json, Value};
    /// use pgxn_meta::meta::*;
    ///
    /// let meta_json = json!({
    ///   "name": "pair",
    ///   "abstract": "A key/value pair data type",
    ///   "version": "0.1.8",
    ///   "maintainers": [
    ///     { "name": "Barrack Obama",  "email": "pogus@example.com" }
    ///   ],
    ///   "license": "PostgreSQL",
    ///   "contents": {
    ///     "extensions": {
    ///       "pair": {
    ///         "sql": "sql/pair.sql",
    ///         "control": "pair.control"
    ///       }
    ///     }
    ///   },
    ///   "meta-spec": { "version": "2.0.0" }
    /// });
    ///
    ///
    /// let meta = Meta::try_from(meta_json);
    /// assert!(meta.is_ok());
    /// let val: Result<Value, Box<dyn Error>> = meta.unwrap().try_into();
    /// assert!(val.is_ok());
    /// ```
    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        let val = serde_json::to_value(meta)?;
        Ok(val)
    }
}

impl TryFrom<&PathBuf> for Meta {
    type Error = Box<dyn Error>;
    /// Reads the `META.json` data from `file` then converts into a [`Meta`].
    /// Returns an error on file error or if the content of `file` is not
    /// valid PGXN `META.json` data.
    fn try_from(file: &PathBuf) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_reader(File::open(file)?)?;
        Meta::try_from(meta)
    }
}

impl TryFrom<&String> for Meta {
    type Error = Box<dyn Error>;
    /// Converts `str` into JSON and then into  a [`Meta`]. Returns an error
    /// if the content of `str` is not valid PGXN `META.json` data.
    fn try_from(str: &String) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_str(str)?;
        Meta::try_from(meta)
    }
}

impl TryFrom<Meta> for String {
    type Error = Box<dyn Error>;
    /// Converts `meta` into a JSON String.
    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        let val = serde_json::to_string(&meta)?;
        Ok(val)
    }
}

#[cfg(test)]
mod tests;
