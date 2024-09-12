/*!
PGXN `META.json` validation and management.

This module provides interfaces to load, validate, and manipulate PGXN
`META.json` files. It supports both the [v1] and [v2] specs.

  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3

*/
use std::{borrow::Borrow, collections::HashMap, error::Error, fs::File, path::PathBuf};

use crate::util;
use relative_path::{RelativePath, RelativePathBuf};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

mod v1;
mod v2;

/// Represents the `meta-spec` object in [`Distribution`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Spec {
    version: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Spec {
    /// Borrows the Spec version.
    pub fn version(&self) -> &Version {
        self.version.borrow()
    }

    /// Borrows the Spec URL.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

/// Maintainer represents an object in the list of `maintainers` in
/// [`Distribution`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Maintainer {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Maintainer {
    /// Borrows the Maintainer name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Borrows the Maintainer email.
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Borrows the Maintainer URL.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Extension {
    /// Borrows the Extension control file location.
    pub fn control(&self) -> &RelativePathBuf {
        self.control.borrow()
    }

    /// Borrows the Extension abstract.
    pub fn abs_tract(&self) -> Option<&str> {
        self.abs_tract.as_deref()
    }

    /// Returns true if the Extension is marked as a trusted language
    /// extension.
    pub fn tle(&self) -> bool {
        self.tle.unwrap_or(false)
    }

    /// Borrows the Extension sql file location.
    pub fn sql(&self) -> &RelativePathBuf {
        self.sql.borrow()
    }

    /// Borrows the Extension doc file location.
    pub fn doc(&self) -> Option<&RelativePath> {
        self.doc.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines a type of module in [`Module`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ModuleType {
    /// Indicates an extension shared library module.
    #[serde(rename = "extension")]
    Extension,
    /// Indicates a hook shared library module.
    #[serde(rename = "hook")]
    Hook,
    /// Indicates a background worker shared library module.
    #[serde(rename = "bgw")]
    Bgw,
}

impl std::fmt::Display for ModuleType {
    /// fmt writes the sting representation of the ModuleType to f.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::Extension => write!(f, "extension"),
            ModuleType::Hook => write!(f, "hook"),
            ModuleType::Bgw => write!(f, "bgw"),
        }
    }
}

/// Defines the values for the `preload` value in [`Module`]s.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Preload {
    /// Indicates a module that should be included in
    /// `shared_preload_libraries` and requires a service restart.
    #[serde(rename = "server")]
    Server,
    /// Indicates a module that can be loaded in a session via
    /// `session_preload_libraries` or `local_preload_libraries`.
    #[serde(rename = "session")]
    Session,
}

impl std::fmt::Display for Preload {
    /// fmt writes the sting representation of the ModuleType to f.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Preload::Server => write!(f, "server"),
            Preload::Session => write!(f, "session"),
        }
    }
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Module {
    /// Borrows the Module type.
    pub fn kind(&self) -> &ModuleType {
        self.kind.borrow()
    }

    /// Borrows the Module abstract.
    pub fn abs_tract(&self) -> Option<&str> {
        self.abs_tract.as_deref()
    }

    /// Borrows the Module preload value.
    pub fn preload(&self) -> Option<&Preload> {
        self.preload.as_ref()
    }

    /// Borrows the Module library file location.
    pub fn lib(&self) -> &RelativePathBuf {
        self.lib.borrow()
    }

    /// Borrows the Module doc file location.
    pub fn doc(&self) -> Option<&RelativePath> {
        self.doc.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl App {
    /// Borrows the App lang field.
    pub fn lang(&self) -> Option<&str> {
        self.lang.as_deref()
    }

    /// Borrows the App abstract.
    pub fn abs_tract(&self) -> Option<&str> {
        self.abs_tract.as_deref()
    }

    /// Borrows the App binary file location.
    pub fn bin(&self) -> &RelativePathBuf {
        self.bin.borrow()
    }

    /// Borrows the App library file location.
    pub fn lib(&self) -> Option<&RelativePath> {
        self.lib.as_deref()
    }

    /// Borrows the App doc file location.
    pub fn doc(&self) -> Option<&RelativePath> {
        self.doc.as_deref()
    }

    /// Borrows the App manual file location.
    pub fn man(&self) -> Option<&RelativePath> {
        self.man.as_deref()
    }

    /// Borrows the App HTML file location.
    pub fn html(&self) -> Option<&RelativePath> {
        self.html.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Represents the contents of a distribution, under `contents` in
/// [`Distribution`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Contents {
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<HashMap<String, Extension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modules: Option<HashMap<String, Module>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apps: Option<HashMap<String, App>>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Contents {
    /// Borrows the Contents extensions object.
    pub fn extensions(&self) -> Option<&HashMap<String, Extension>> {
        self.extensions.as_ref()
    }

    /// Borrows the Contents modules object.
    pub fn modules(&self) -> Option<&HashMap<String, Module>> {
        self.modules.as_ref()
    }

    /// Borrows the Contents apps object.
    pub fn apps(&self) -> Option<&HashMap<String, App>> {
        self.apps.as_ref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Represents the classifications of a distribution, under `classifications`
/// in [`Distribution`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Classifications {
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    categories: Option<Vec<String>>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Classifications {
    /// Borrows the Classifications tags.
    pub fn tags(&self) -> Option<&[String]> {
        self.tags.as_deref()
    }

    /// Borrows the Classifications categories.
    pub fn categories(&self) -> Option<&[String]> {
        self.categories.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Represents Postgres requirements under `postgres` in [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Postgres {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<Vec<String>>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Postgres {
    /// Borrows the Postgres version.
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Borrows the Postgres with field.
    pub fn with(&self) -> Option<&[String]> {
        self.with.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Represents the name of a build pipeline under `pipeline` in
/// [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Pipeline {
    /// PGXS
    #[serde(rename = "pgxs")]
    Pgxs,
    /// Meson
    #[serde(rename = "meson")]
    Meson,
    /// pgrx
    #[serde(rename = "pgrx")]
    Pgrx,
    /// Autoconf
    #[serde(rename = "autoconf")]
    Autoconf,
    /// cmake
    #[serde(rename = "cmake")]
    Cmake,
}

impl std::fmt::Display for Pipeline {
    /// fmt writes the sting representation of the Pipeline to f.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pipeline::Pgxs => write!(f, "pgxs"),
            Pipeline::Meson => write!(f, "meson"),
            Pipeline::Pgrx => write!(f, "pgrx"),
            Pipeline::Autoconf => write!(f, "autoconf"),
            Pipeline::Cmake => write!(f, "cmake"),
        }
    }
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

impl std::fmt::Display for VersionRange {
    /// fmt writes the sting representation of the Pipeline to f.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionRange::Integer(int) => write!(f, "{int}"),
            VersionRange::String(str) => write!(f, "{str}"),
        }
    }
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Phase {
    /// Borrows the Phase requires object.
    pub fn requires(&self) -> Option<&HashMap<String, VersionRange>> {
        self.requires.as_ref()
    }

    /// Borrows the Phase recommends object.
    pub fn recommends(&self) -> Option<&HashMap<String, VersionRange>> {
        self.recommends.as_ref()
    }

    /// Borrows the Phase suggests object.
    pub fn suggests(&self) -> Option<&HashMap<String, VersionRange>> {
        self.suggests.as_ref()
    }

    /// Borrows the Phase conflicts object.
    pub fn conflicts(&self) -> Option<&HashMap<String, VersionRange>> {
        self.conflicts.as_ref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Packages {
    /// Borrows the Packages configure object.
    pub fn configure(&self) -> Option<&Phase> {
        self.configure.as_ref()
    }

    /// Borrows the Packages build object.
    pub fn build(&self) -> Option<&Phase> {
        self.build.as_ref()
    }

    /// Borrows the Packages test object.
    pub fn test(&self) -> Option<&Phase> {
        self.test.as_ref()
    }

    /// Borrows the Packages run object.
    pub fn run(&self) -> Option<&Phase> {
        self.run.as_ref()
    }

    /// Borrows the Packages develop object.
    pub fn develop(&self) -> Option<&Phase> {
        self.develop.as_ref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines dependency variations under `variations`in  [`Dependencies`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Variations {
    #[serde(rename = "where")]
    wheres: Dependencies,
    dependencies: Dependencies,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Variations {
    /// Borrows the Variations wheres field.
    pub fn wheres(&self) -> &Dependencies {
        self.wheres.borrow()
    }

    /// Borrows the Variations dependencies field.
    pub fn dependencies(&self) -> &Dependencies {
        self.dependencies.borrow()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines the distribution dependencies under `dependencies` in [`Distribution`].
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Dependencies {
    /// Borrows the Dependencies platforms object.
    pub fn platforms(&self) -> Option<&[String]> {
        self.platforms.as_deref()
    }

    /// Borrows the Dependencies postgres object.
    pub fn postgres(&self) -> Option<&Postgres> {
        self.postgres.as_ref()
    }

    /// Borrows the Dependencies pipeline value.
    pub fn pipeline(&self) -> Option<&Pipeline> {
        self.pipeline.as_ref()
    }

    /// Borrows the Dependencies packages object.
    pub fn packages(&self) -> Option<&Packages> {
        self.packages.as_ref()
    }

    /// Borrows the Dependencies variations collection.
    pub fn variations(&self) -> Option<&[Variations]> {
        self.variations.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines the badges under `badges` in [`Resources`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Badge {
    src: String,
    alt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(flatten)]
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Badge {
    /// Borrows the Badge src URL
    pub fn src(&self) -> &str {
        self.src.as_str()
    }

    /// Borrows the Badge alt text.
    pub fn alt(&self) -> &str {
        self.alt.as_str()
    }

    /// Borrows the Badge link URL.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines the resources under `resources` in [`Distribution`].
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Resources {
    /// Borrows the Resources homepage URL.
    pub fn homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    /// Borrows the Resources issues URL.
    pub fn issues(&self) -> Option<&str> {
        self.issues.as_deref()
    }

    /// Borrows the Resources repository URL.
    pub fn repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    /// Borrows the Resources docs URL.
    pub fn docs(&self) -> Option<&str> {
        self.docs.as_deref()
    }

    /// Borrows the Resources support URL.
    pub fn support(&self) -> Option<&str> {
        self.support.as_deref()
    }

    /// Borrows the Resources badges objects.
    pub fn badges(&self) -> Option<&[Badge]> {
        self.badges.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/// Defines the artifacts in the array under `artifacts` in [`Distribution`].
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

impl Artifact {
    /// Borrows the Artifact URL.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Borrows the Artifact type.
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }

    /// Borrows the Artifact platform property.
    pub fn platform(&self) -> Option<&str> {
        self.platform.as_deref()
    }

    /// Borrows the Artifact sha256 property.
    pub fn sha256(&self) -> Option<&str> {
        self.sha256.as_deref()
    }

    /// Borrows the Artifact sha512 property.
    pub fn sha512(&self) -> Option<&str> {
        self.sha512.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

/**
Represents the `META.json` data from a PGXN distribution.

Use the [TryFrom] traits to load a Distribution object from a file, string, or
[serde_json::Value]. These constructors validate the `META.json` data against
a JSON schema, provided by the [crate::valid] package. Once loaded, the data
should never need to be modified; hence the read-only accessors to its
contents.

For cases where PGXN `META.json` data does need to be modified, use the
[`TryFrom<&[&Value]>`](#impl-TryFrom%3C%26%5B%26Value%5D%3E-for-Distribution) trait to
merge merge one or more [RFC 7396] patches.

  [RFC 7396]: https://www.rfc-editor.org/rfc/rfc7396.html
*/
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Distribution {
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
    #[serde(deserialize_with = "deserialize_custom_properties")]
    custom_props: HashMap<String, Value>,
}

/// Deserializes fields starting with `X_` or `x_` into a HashMap.
pub fn deserialize_custom_properties<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;

    Ok(map
        .into_iter()
        .filter(|(key, _value)| key.starts_with("x_") || key.starts_with("X_"))
        .collect())
}

impl Distribution {
    /// Deserializes `meta`, which contains PGXN `version` metadata, into a
    /// [`Distribution`].
    fn from_version(version: u8, meta: Value) -> Result<Self, Box<dyn Error>> {
        match version {
            1 => v1::from_value(meta),
            2 => v2::from_value(meta),
            _ => Err(Box::from(format!("Unknown meta version {version}"))),
        }
    }

    /// Borrows the Distribution name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Borrows the Distribution version.
    pub fn version(&self) -> &Version {
        self.version.borrow()
    }

    /// Borrows the Distribution abstract.
    pub fn abs_tract(&self) -> &str {
        self.abs_tract.as_str()
    }

    /// Borrows the Distribution description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Borrows the Distribution producer.
    pub fn producer(&self) -> Option<&str> {
        self.producer.as_deref()
    }

    /// Borrows the Distribution license string.
    pub fn license(&self) -> &str {
        self.license.as_str()
    }

    /// Borrows the Distribution meta spec object.
    pub fn spec(&self) -> &Spec {
        self.spec.borrow()
    }

    /// Borrows the Distribution maintainers collection.
    pub fn maintainers(&self) -> &[Maintainer] {
        self.maintainers.borrow()
    }

    /// Borrows the Dependencies classifications object.
    pub fn classifications(&self) -> Option<&Classifications> {
        self.classifications.as_ref()
    }

    /// Borrows the Distribution contents object.
    pub fn contents(&self) -> &Contents {
        self.contents.borrow()
    }

    /// Borrows the Distribution ignore list.
    pub fn ignore(&self) -> Option<&[String]> {
        self.ignore.as_deref()
    }

    /// Borrows the Distribution meta dependencies object.
    pub fn dependencies(&self) -> Option<&Dependencies> {
        self.dependencies.as_ref()
    }

    /// Borrows the Distribution meta resources object.
    pub fn resources(&self) -> Option<&Resources> {
        self.resources.as_ref()
    }

    /// Borrows the Distribution artifacts list.
    pub fn artifacts(&self) -> Option<&[Artifact]> {
        self.artifacts.as_deref()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.custom_props.borrow()
    }
}

impl TryFrom<Value> for Distribution {
    type Error = Box<dyn Error>;
    /// Converts the PGXN `META.json` data from `meta` into a
    /// [`Distribution`]. Returns an error if `meta` is invalid.
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
    /// let meta = Distribution::try_from(meta_json);
    /// assert!(meta.is_ok());
    /// ```
    fn try_from(meta: Value) -> Result<Self, Self::Error> {
        // Make sure it's valid.
        let mut validator = crate::valid::Validator::new();
        let version = match validator.validate(&meta) {
            Err(e) => return Err(Box::from(e.to_string())),
            Ok(v) => v,
        };
        Distribution::from_version(version, meta)
    }
}

impl TryFrom<&[&Value]> for Distribution {
    type Error = Box<dyn Error>;
    /// Merge multiple PGXN `META.json` data from `meta` into a
    /// [`Distribution`]. Returns an error if `meta` is invalid.
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
    /// let meta = Distribution::try_from(&all_meta[..]);
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
        Distribution::from_version(2, v2)
    }
}

impl TryFrom<Distribution> for Value {
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
    /// let meta = Distribution::try_from(meta_json);
    /// assert!(meta.is_ok());
    /// let val: Result<Value, Box<dyn Error>> = meta.unwrap().try_into();
    /// assert!(val.is_ok());
    /// ```
    fn try_from(meta: Distribution) -> Result<Self, Self::Error> {
        let val = serde_json::to_value(meta)?;
        Ok(val)
    }
}

impl TryFrom<&PathBuf> for Distribution {
    type Error = Box<dyn Error>;
    /// Reads the `META.json` data from `file` then converts into a
    /// [`Distribution`]. Returns an error on file error or if the content of
    /// `file` is not valid PGXN `META.json` data.
    fn try_from(file: &PathBuf) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_reader(File::open(file)?)?;
        Distribution::try_from(meta)
    }
}

impl TryFrom<&String> for Distribution {
    type Error = Box<dyn Error>;
    /// Converts `str` into JSON and then into  a [`Distribution`]. Returns an
    /// error if the content of `str` is not valid PGXN `META.json` data.
    fn try_from(str: &String) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_str(str)?;
        Distribution::try_from(meta)
    }
}

impl TryFrom<Distribution> for String {
    type Error = Box<dyn Error>;
    /// Converts `meta` into a JSON String.
    fn try_from(meta: Distribution) -> Result<Self, Self::Error> {
        let val = serde_json::to_string(&meta)?;
        Ok(val)
    }
}

#[cfg(test)]
mod tests;
