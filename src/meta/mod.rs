use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use relative_path::RelativePathBuf;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod v1;
mod v2;

fn meta_url() -> String {
    "https://rfcs.pgxn.org/0003-meta-spec-v2.html".to_string()
}

/// Represents the `meta-spec` object in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Spec {
    version: String,
    #[serde(default = "meta_url")]
    url: String,
}

/// Maintainer represents an object in the list of `maintainers` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Maintainer {
    name: String,
    email: Option<String>,
    url: Option<String>,
}

/// Describes an extension in under `extensions` in [`Contents`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Extension {
    control: RelativePathBuf,
    #[serde(rename = "abstract")]
    abs_tract: Option<String>,
    tle: Option<bool>,
    sql: RelativePathBuf,
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
    abs_tract: Option<String>,
    preload: Option<Preload>,
    lib: RelativePathBuf,
    doc: Option<RelativePathBuf>,
}

/// Represents an app under `apps` in [`Contents`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct App {
    lang: Option<String>,
    #[serde(rename = "abstract")]
    abs_tract: Option<String>,
    bin: RelativePathBuf,
    doc: Option<RelativePathBuf>,
    lib: Option<RelativePathBuf>,
    man: Option<RelativePathBuf>,
    html: Option<RelativePathBuf>,
}

/// Represents the contents of a distribution, under `contents` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Contents {
    extensions: Option<HashMap<String, Extension>>,
    modules: Option<HashMap<String, Module>>,
    apps: Option<HashMap<String, App>>,
}

/// Represents the classifications of a distribution, under `classifications`
/// in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Classifications {
    tags: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}

/// Represents Postgres requirements under `postgres` in [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Postgres {
    version: String,
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
    requires: Option<HashMap<String, VersionRange>>,
    recommends: Option<HashMap<String, VersionRange>>,
    suggests: Option<HashMap<String, VersionRange>>,
    conflicts: Option<HashMap<String, VersionRange>>,
}

/// Defines package dependencies for build phases under `packages` in
/// [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Packages {
    configure: Option<Phase>,
    build: Option<Phase>,
    test: Option<Phase>,
    run: Option<Phase>,
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
    platforms: Option<Vec<String>>,
    postgres: Option<Postgres>,
    pipeline: Option<Pipeline>,
    packages: Option<Packages>,
    variations: Option<Vec<Variations>>,
}

/// Defines the resources under `resources` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Resources {
    homepage: Option<String>,
    issues: Option<String>,
    repository: Option<String>,
    docs: Option<String>,
    support: Option<String>,
}

/// Defines the artifacts in the array under `artifacts` in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Artifact {
    url: String,
    #[serde(rename = "type")]
    kind: String,
    platform: Option<String>,
    sha256: Option<String>,
    sha512: Option<String>,
}

/// Represents a complete PGXN Meta definition.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Meta {
    name: String,
    version: Version,
    #[serde(rename = "abstract")]
    abs_tract: String,
    description: Option<String>,
    producer: Option<String>,
    license: String, // use spdx::Expression.
    #[serde(rename = "meta-spec")]
    spec: Spec,
    maintainers: Vec<Maintainer>,
    classifications: Option<Classifications>,
    contents: Contents,
    ignore: Option<Vec<String>>,
    dependencies: Option<Dependencies>,
    resources: Option<Resources>,
    artifacts: Option<Vec<Artifact>>,
}

impl Meta {
    fn from_version(version: u8, meta: Value) -> Result<Self, Box<dyn Error>> {
        match version {
            1 => v1::from_value(meta),
            2 => v2::from_value(meta),
            _ => Err(Box::from(format!("Unknown meta version {version}"))),
        }
    }
}

impl TryFrom<Value> for Meta {
    type Error = Box<dyn Error>;
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

impl TryFrom<&PathBuf> for Meta {
    type Error = Box<dyn Error>;
    fn try_from(file: &PathBuf) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_reader(File::open(file)?)?;
        Meta::try_from(meta)
    }
}

impl TryFrom<&String> for Meta {
    type Error = Box<dyn Error>;
    fn try_from(str: &String) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_str(str)?;
        Meta::try_from(meta)
    }
}

#[cfg(test)]
mod tests;
