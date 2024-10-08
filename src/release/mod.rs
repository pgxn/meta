/*!
PGXN release `META.json` validation and management.

This module provides interfaces to load, validate, and manipulate PGXN release
`META.json` files. PGXN adds release metadata to distribution-provided
[v1] and [v2] `META.json` data to identify the user who made a release, the
timestamp, hash digests of the release file. In [v2], it also includes a
download URI and a private key signature.

Use [`Distribution`] to validate the `META.json` file included in a
distribution.

It supports both the [v1] and [v2] specs.

  [v1]: https://rfcs.pgxn.org/0001-meta-spec-v1.html
  [v2]: https://github.com/pgxn/rfcs/pull/3

*/

use crate::dist::*;
use crate::util;

use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{borrow::Borrow, collections::HashMap, error::Error, fs::File, path::Path};

mod v1;
mod v2;

/// Digests represents Hash digests for a file that can be used to verify its
/// integrity.
#[serde_with::serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Digests {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    sha1: Option<[u8; 20]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    sha256: Option<[u8; 32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    sha512: Option<[u8; 64]>,
}

impl Digests {
    /// Borrows the SHA-1 hash.
    pub fn sha1(&self) -> Option<&[u8; 20]> {
        self.sha1.as_ref()
    }

    /// Borrows the SHA-256 hash.
    pub fn sha256(&self) -> Option<&[u8; 32]> {
        self.sha256.as_ref()
    }

    /// Borrows the SHA-256 hash.
    pub fn sha512(&self) -> Option<&[u8; 64]> {
        self.sha512.as_ref()
    }
}

/// ReleasePayload represents release metadata populated by PGXN.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReleasePayload {
    user: String,
    date: DateTime<Utc>,
    uri: String,
    digests: Digests,
}

impl ReleasePayload {
    /// Borrows the release user name.
    pub fn user(&self) -> &str {
        self.user.as_str()
    }

    /// Borrows the release date.
    pub fn date(&self) -> &DateTime<Utc> {
        self.date.borrow()
    }

    /// Borrows the release URI.
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    /// Borrows the release digests.
    pub fn digests(&self) -> &Digests {
        self.digests.borrow()
    }
}

/// ReleaseJws represents JSON Web Signature release metadata populated by
/// PGXN.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReleaseJws {
    headers: Vec<String>,
    signatures: Vec<String>,
    payload: ReleasePayload,
}

impl ReleaseJws {
    /// Borrows the signature headers.
    pub fn headers(&self) -> &[String] {
        self.headers.as_slice()
    }

    /// Borrows the signatures.
    pub fn signatures(&self) -> &[String] {
        self.signatures.as_slice()
    }

    /// Borrows the release payload.
    pub fn payload(&self) -> &ReleasePayload {
        self.payload.borrow()
    }
}

/**

Represents metadata for a PGXN release, which is the same as [`Distribution`]
plus [`ReleaseJws`] that contains signed metadata about the release to PGXN.

*/
#[derive(Serialize, PartialEq, Debug)]
pub struct Release {
    #[serde(flatten)]
    dist: Distribution,
    certs: HashMap<String, Value>,
    #[serde(skip_serializing)]
    release: ReleasePayload,
}

impl<'de> Deserialize<'de> for Release {
    /// deserialize deserializes a Release. Required to transparently
    /// deserialize and validate the `release` field from `certs`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize into a struct with just the dist and certs.
        #[derive(Deserialize)]
        struct ReleaseInitial {
            #[serde(flatten)]
            dist: Distribution,
            certs: HashMap<String, Value>,
        }
        let rel = ReleaseInitial::deserialize(deserializer)?;

        // Fetch the pgxn release JWS from the certs object.
        let Some(Value::Object(jws)) = rel.certs.get("pgxn") else {
            return Err(de::Error::custom("invalid or missing pgxn release data"));
        };

        // XXX Use the jose_jws crate to validate signature here.

        // Fetch the JWS payload.
        let Some(Value::String(b64)) = jws.get("payload") else {
            return Err(de::Error::custom("missing or invalid pgxn payload"));
        };

        // Decode the payload from base64-encoded JSON.
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
        let json = URL_SAFE_NO_PAD.decode(b64).map_err(de::Error::custom)?;

        // Parse and validate the JSON.
        let pay = serde_json::from_slice(&json).map_err(de::Error::custom)?;
        let mut v = crate::valid::Validator::new();
        v.validate_payload(&pay).map_err(de::Error::custom)?;

        // Decode the ReleasePayload and return the complete Release struct.
        Ok(Release {
            dist: rel.dist,
            certs: rel.certs,
            release: serde_json::from_value(pay).map_err(de::Error::custom)?,
        })
    }
}

impl Release {
    // It would be nice to use [delegation] at some point instead of
    // copy/pasting all the Distribution methods, but this will do for now.
    // [delegation]: https://github.com/rust-lang/rfcs/pull/3530

    /// Deserializes `meta`, which contains PGXN distribution release
    /// metadata, into a [`Release`].
    fn from_version(version: u8, meta: Value) -> Result<Self, Box<dyn Error>> {
        match version {
            1 => v1::from_value(meta),
            2 => v2::from_value(meta),
            _ => Err(Box::from(format!("Unknown meta version {version}"))),
        }

        // XXX: Add signature validation.
    }

    /// Loads the release `META.json` data from `file` then converts into a
    /// [`Release`]. Returns an error on file error or if the content of
    /// `file` is not valid PGXN `META.json` data.
    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self, Box<dyn Error>> {
        let meta: Value = serde_json::from_reader(File::open(file)?)?;
        meta.try_into()
    }

    /// Borrows the Distribution name.
    pub fn name(&self) -> &str {
        self.dist.name()
    }

    /// Borrows the Distribution version.
    pub fn version(&self) -> &semver::Version {
        self.dist.version()
    }

    /// Borrows the Distribution abstract.
    pub fn abs_tract(&self) -> &str {
        self.dist.abs_tract()
    }

    /// Borrows the Distribution description.
    pub fn description(&self) -> Option<&str> {
        self.dist.description()
    }

    /// Borrows the Distribution producer.
    pub fn producer(&self) -> Option<&str> {
        self.dist.producer()
    }

    /// Borrows the Distribution license string.
    pub fn license(&self) -> &str {
        self.dist.license()
    }

    /// Borrows the Distribution meta spec object.
    pub fn spec(&self) -> &Spec {
        self.dist.spec()
    }

    /// Borrows the Distribution maintainers collection.
    pub fn maintainers(&self) -> &[Maintainer] {
        self.dist.maintainers()
    }

    /// Borrows the Dependencies classifications object.
    pub fn classifications(&self) -> Option<&Classifications> {
        self.dist.classifications()
    }

    /// Borrows the Distribution contents object.
    pub fn contents(&self) -> &Contents {
        self.dist.contents()
    }

    /// Borrows the Distribution ignore list.
    pub fn ignore(&self) -> Option<&[String]> {
        self.dist.ignore()
    }

    /// Borrows the Distribution meta dependencies object.
    pub fn dependencies(&self) -> Option<&Dependencies> {
        self.dist.dependencies()
    }

    /// Borrows the Distribution meta resources object.
    pub fn resources(&self) -> Option<&Resources> {
        self.dist.resources()
    }

    /// Borrows the Distribution artifacts list.
    pub fn artifacts(&self) -> Option<&[Artifact]> {
        self.dist.artifacts()
    }

    /// Borrows the Distribution release metadata.
    pub fn release(&self) -> &ReleasePayload {
        self.release.borrow()
    }

    /// Borrows the Distribution certifications.
    pub fn certs(&self) -> &HashMap<String, Value> {
        self.certs.borrow()
    }

    /// Borrows the custom_props object, which holds any `x_` or `X_`
    /// properties
    pub fn custom_props(&self) -> &HashMap<String, Value> {
        self.dist.custom_props()
    }
}

impl TryFrom<Value> for Release {
    type Error = Box<dyn Error>;
    /// Converts the PGXN release `META.json` data from `meta` into a
    /// [`Release`]. Returns an error if `meta` is invalid.
    ///
    /// # Example
    ///
    /// ``` rust
    /// # use std::error::Error;
    /// use serde_json::json;
    /// use pgxn_meta::release::*;
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
    ///   "meta-spec": { "version": "2.0.0" },
    ///   "certs": {
    ///     "pgxn": {
    ///       "payload": "eyJ1c2VyIjoidGhlb3J5IiwiZGF0ZSI6IjIwMjQtMDktMTNUMTc6MzI6NTVaIiwidXJpIjoiZGlzdC9wYWlyLzAuMS43L3BhaXItMC4xLjcuemlwIiwiZGlnZXN0cyI6eyJzaGE1MTIiOiJiMzUzYjVhODJiM2I1NGU5NWY0YTI4NTllN2EyYmQwNjQ4YWJjYjM1YTdjMzYxMmIxMjZjMmM3NTQzOGZjMmY4ZThlZTFmMTllNjFmMzBmYTU0ZDdiYjY0YmNmMjE3ZWQxMjY0NzIyYjQ5N2JjYjYxM2Y4MmQ3ODc1MTUxNWI2NyJ9fQ",
    ///       "signature": "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
    ///     }
    ///   }
    /// });
    ///
    ///
    /// let meta = Release::try_from(meta_json);
    /// assert!(meta.is_ok(), "{:?}", meta);
    /// ```
    fn try_from(meta: Value) -> Result<Self, Self::Error> {
        // Make sure it's valid.
        let mut validator = crate::valid::Validator::new();
        let version = match validator.validate_release(&meta) {
            Err(e) => return Err(Box::from(e.to_string())),
            Ok(v) => v,
        };
        Release::from_version(version, meta)
    }
}

impl TryFrom<&[&Value]> for Release {
    type Error = Box<dyn Error>;
    /// Merge multiple PGXN release `META.json` data from `meta` into a
    /// [`Release`]. Returns an error if `meta` is invalid.
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
    /// use pgxn_meta::release::*;
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
    ///   "meta-spec": { "version": "2.0.0" },
    ///   "certs": {
    ///     "pgxn": {
    ///       "payload": "eyJ1c2VyIjoidGhlb3J5IiwiZGF0ZSI6IjIwMjQtMDktMTNUMTc6MzI6NTVaIiwidXJpIjoiZGlzdC9wYWlyLzAuMS43L3BhaXItMC4xLjcuemlwIiwiZGlnZXN0cyI6eyJzaGE1MTIiOiJiMzUzYjVhODJiM2I1NGU5NWY0YTI4NTllN2EyYmQwNjQ4YWJjYjM1YTdjMzYxMmIxMjZjMmM3NTQzOGZjMmY4ZThlZTFmMTllNjFmMzBmYTU0ZDdiYjY0YmNmMjE3ZWQxMjY0NzIyYjQ5N2JjYjYxM2Y4MmQ3ODc1MTUxNWI2NyJ9fQ",
    ///       "signature": "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
    ///     }
    ///   }
    /// });
    ///
    /// let patch = json!({"license": "MIT"});
    /// let all_meta = [&meta_json, &patch];
    ///
    /// let meta = Release::try_from(&all_meta[..]);
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
        validator.validate_release(&v2).map_err(|e| e.to_string())?;
        Release::from_version(2, v2)
    }
}

impl TryFrom<Release> for Value {
    type Error = Box<dyn Error>;
    /// Converts PGXN release `meta` into a [serde_json::Value].
    ///
    /// # Example
    ///
    /// ``` rust
    /// # use std::error::Error;
    /// use serde_json::{json, Value};
    /// use pgxn_meta::release::*;
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
    ///   "meta-spec": { "version": "2.0.0" },
    ///   "certs": {
    ///     "pgxn": {
    ///       "payload": "eyJ1c2VyIjoidGhlb3J5IiwiZGF0ZSI6IjIwMjQtMDktMTNUMTc6MzI6NTVaIiwidXJpIjoiZGlzdC9wYWlyLzAuMS43L3BhaXItMC4xLjcuemlwIiwiZGlnZXN0cyI6eyJzaGE1MTIiOiJiMzUzYjVhODJiM2I1NGU5NWY0YTI4NTllN2EyYmQwNjQ4YWJjYjM1YTdjMzYxMmIxMjZjMmM3NTQzOGZjMmY4ZThlZTFmMTllNjFmMzBmYTU0ZDdiYjY0YmNmMjE3ZWQxMjY0NzIyYjQ5N2JjYjYxM2Y4MmQ3ODc1MTUxNWI2NyJ9fQ",
    ///       "signature": "DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q"
    ///     }
    ///   }
    /// });
    ///
    ///
    /// let meta = Release::try_from(meta_json);
    /// assert!(meta.is_ok());
    /// let val: Result<Value, Box<dyn Error>> = meta.unwrap().try_into();
    /// assert!(val.is_ok());
    /// ```
    fn try_from(meta: Release) -> Result<Self, Self::Error> {
        let val = serde_json::to_value(meta)?;
        Ok(val)
    }
}

impl TryFrom<&String> for Release {
    type Error = Box<dyn Error>;
    /// Converts `str` into JSON and then into a [`Release`]. Returns an
    /// error if the content of `str` is not valid PGXN `META.json` data.
    fn try_from(str: &String) -> Result<Self, Self::Error> {
        let meta: Value = serde_json::from_str(str)?;
        meta.try_into()
    }
}

impl TryFrom<Release> for String {
    type Error = Box<dyn Error>;
    /// Converts `meta` into a JSON String.
    fn try_from(meta: Release) -> Result<Self, Self::Error> {
        let val = serde_json::to_string(&meta)?;
        Ok(val)
    }
}

#[cfg(test)]
mod tests;
