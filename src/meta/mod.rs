use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use relative_path::RelativePathBuf;
use semver::Version;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Represents the classifications of a distribution, under `classifications`
/// in [`Meta`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Classifications {
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Vec<String>>,
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Represents Postgres requirements under `postgres` in [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Postgres {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<Vec<String>>,
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Defines dependency variations under `variations`in  [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Variations {
    #[serde(rename = "where")]
    wheres: Box<Dependencies>,
    dependencies: Box<Dependencies>,
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Defines the badges under `badges` in [`Resources`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Badge {
    src: String,
    alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Represents a complete PGXN Meta definition.
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
    #[serde(flatten)]
    #[serde(serialize_with = "serialize_custom_properties")]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Deserializes extra fields starting with `X_` or `x_` into the `custom_properties` HashMap.
pub fn deserialize_custom_properties<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;

    Ok(map
        .into_iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("x_")
                .or(key.strip_prefix("X_"))
                .map(|key| (key.to_string(), value))
        })
        .collect())
}

/// Serializes the `custom_properties` HashMap into fields starting with `X_` or `x_`
pub fn serialize_custom_properties<S>(
    custom_props: &HashMap<String, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(custom_props.len()))?;
    for (key, value) in custom_props {
        let is_uppercase = key.chars().next().map(char::is_uppercase).unwrap_or(false);

        if is_uppercase {
            map.serialize_entry(&format_args!("X_{key}"), value)?;
        } else {
            map.serialize_entry(&format_args!("x_{key}"), value)?;
        }
    }

    map.end()
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

impl TryFrom<Meta> for Value {
    type Error = Box<dyn Error>;
    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        let val = serde_json::to_value(meta)?;
        Ok(val)
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

impl TryFrom<Meta> for String {
    type Error = Box<dyn Error>;
    fn try_from(meta: Meta) -> Result<Self, Self::Error> {
        let val = serde_json::to_string(&meta)?;
        Ok(val)
    }
}

#[cfg(test)]
mod tests;
