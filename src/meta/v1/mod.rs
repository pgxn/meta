use super::*;

use email_address::EmailAddress;
use serde_json::{json, Map, Value};
use std::{error::Error, str::FromStr};

/// to_v2 parses v1, which contains PGXN v1 metadata, into a JSON object
/// containing valid PGXN v2 metadata.
pub fn to_v2(v1: &Value) -> Result<Value, Box<dyn Error>> {
    // Copy common fields.
    let mut v2 = v1_to_v2_common(v1);

    // Convert maintainer.
    v2.insert("maintainers".to_string(), v1_to_v2_maintainers(v1)?);

    // Convert license.
    v2.insert("license".to_string(), v1_to_v2_license(v1)?);

    // Convert provides to contents.
    v2.insert("contents".to_string(), v1_to_v2_contents(v1)?);

    // Convert tags to classifications.
    if let Some(val) = v1_to_v2_classifications(v1) {
        v2.insert("classifications".to_string(), val);
    }

    // Convert no_index to ignore.
    if let Some(val) = v1_to_v2_ignore(v1) {
        v2.insert("ignore".to_string(), val);
    }

    // Convert prereqs to dependencies.
    if let Some(val) = v1_to_v2_dependencies(v1) {
        v2.insert("dependencies".to_string(), val);
    }

    // resources
    if let Some(val) = v1_to_v2_resources(v1) {
        v2.insert("resources".to_string(), val);
    }

    Ok(Value::Object(v2))
}

/// from_value parses v1, which contains PGXN v1 metadata, into a
/// [`Distribution`] object containing valid PGXN v2 metadata.
pub fn from_value(v1: Value) -> Result<Distribution, Box<dyn Error>> {
    Distribution::try_from(to_v2(&v1)?)
}

/// v1_to_v2_common sets up a new v2 map with compatible fields copied from v1
/// and the `meta-spec` field set appropriately.
fn v1_to_v2_common(v1: &Value) -> Map<String, Value> {
    let mut v2 = Map::new();

    // Copy fields unchanged from v1.
    for (k1, k2) in [
        ("name", "name"),
        ("abstract", "abstract"),
        ("description", "description"),
        ("version", "version"),
        ("generated_by", "producer"),
    ] {
        if let Some(v) = v1.get(k1) {
            v2.insert(k2.to_string(), v.clone());
        }
    }

    // Copy custom properties.
    v1_value_to_v2_custom_props(v1, &mut v2);

    // Set the meta-spec.
    let mut spec = Map::new();
    spec.insert("version".to_string(), json!("2.0.0"));
    spec.insert(
        "url".to_string(),
        json!("https://rfcs.pgxn.org/0003-meta-spec-v2.html"),
    );
    if let Some(v1_spec) = v1.get("meta-spec") {
        v1_value_to_v2_custom_props(v1_spec, &mut spec);
    }
    v2.insert("meta-spec".to_string(), Value::Object(spec));

    v2
}

// v1_value_to_v2_custom_props copies all custom properties from v1 to v2.
fn v1_value_to_v2_custom_props(v1: &Value, v2: &mut Map<String, Value>) {
    if let Some(obj) = v1.as_object() {
        v1_to_v2_custom_props(obj, v2);
    }
}

// v1_to_v2_custom_props copies all custom properties from v1 to v2.
fn v1_to_v2_custom_props(v1: &Map<String, Value>, v2: &mut Map<String, Value>) {
    for (k, v) in v1
        .into_iter()
        .filter(|(key, _)| key.starts_with("x_") || key.starts_with("X_"))
    {
        v2.insert(k.to_string(), v.clone());
    }
}

/// v1_to_v2_maintainers clones maintainer data in v1 into the v2 format. It
/// attempts to parse an email address from each maintainer in v1; if there is
/// no email address, it sets `url` the value in `resources.homepage`, if
/// present, and otherwise to `https://pgxn.org`.
fn v1_to_v2_maintainers(v1: &Value) -> Result<Value, Box<dyn Error>> {
    if let Some(maintainer) = v1.get("maintainer") {
        return match maintainer {
            Value::Array(list) => parse_v1_maintainers(v1, list),
            Value::String(_) => {
                let list = vec![maintainer.clone()];
                parse_v1_maintainers(v1, &list)
            }
            _ => Err(Box::from(format!("Invalid v1 maintainer: {maintainer}"))),
        };
    }
    Err(Box::from("maintainer property missing"))
}

/// parse_v1_maintainers parses list for a list of v1 maintainer strings and
/// returns a list of v2 maintainer objects. For each v1 maintainer string, if
/// contains an email address, the address and email display name will be used
/// in for the maintainer `email` and `name` properties, respectively.
/// Otherwise the string will be saved as the maintainer `name` and the `url`
/// set to either the `homepage` in the `resources` object in `v1`, or else
/// `https://pgxn.org`.
fn parse_v1_maintainers(v1: &Value, list: &[Value]) -> Result<Value, Box<dyn Error>> {
    let mut new_list: Vec<Value> = Vec::with_capacity(list.len());
    for v in list {
        if let Some(str) = v.as_str() {
            if let Ok(email) = EmailAddress::from_str(str) {
                new_list.push(json!({
                    "name": match email.display_part() {
                        "" => str,
                        d => d,
                    },
                    "email": email.email(),
                }));
            } else {
                // No email address found. Try using resources.homepage.
                const FALLBACK_URL: &str = "https://pgxn.org";
                let url = match v1.get("resources") {
                    Some(Value::Object(resources)) => match resources.get("homepage") {
                        Some(Value::String(home)) => home.to_string(),
                        _ => FALLBACK_URL.to_string(),
                    },
                    _ => FALLBACK_URL.to_string(),
                };
                new_list.push(json!({"name": str, "url": url}));
            }
        } else {
            return Err(Box::from(format!("Invalid v1 maintainer: {v}")));
        }
    }

    Ok(Value::Array(new_list))
}

/// v1_to_v2_license converts the value in `v1.license` into a v2 license
/// expression:
///
/// *   If `v1.license` is a string, its value is converted to an SPDX license
///     string by [license_expression_for], and returns an error if the
///     license cannot be converted.
/// *   If `v1.license` is an array, each value is converted to an SPDX
///     license string by [license_expression_for], and returns an error if
///     any license cannot be converted. Otherwise, the resulting list of
///     license strings is `OR`ed into a license expression.
/// *   If `v1.license` is an object, each key/value pair is converted to an
///     SPDX license based on its key. The list of supported keys is derived
///     from those used on PGXN, and all should be mapped to valid values. If
///     not, an error will be returned. Otherwise, the resulting list of
///     license strings is `OR`ed into a license expression.
fn v1_to_v2_license(v1: &Value) -> Result<Value, Box<dyn Error>> {
    if let Some(license) = v1.get("license") {
        return match license {
            Value::String(l) => {
                if let Some(name) = license_expression_for(l.as_str()) {
                    return Ok(Value::String(name.to_string()));
                }
                Err(Box::from(format!("Invalid v1 license: {license}")))
            }
            Value::Array(list) => {
                // https://users.rust-lang.org/t/replace-elements-of-a-vector-as-a-function-of-previous-values/101618/6
                let mut v = Vec::with_capacity(list.len());
                for ln in list {
                    match ln {
                        Value::String(s) => {
                            match license_expression_for(s.as_str()) {
                                Some(name) => v.push(name.to_string()),
                                None => return Err(Box::from(format!("Invalid v1 license: {ln}"))),
                            };
                        }
                        _ => return Err(Box::from(format!("Invalid v1 license: {ln}"))),
                    };
                }
                return Ok(Value::String(v.join(" OR ")));
            }
            Value::Object(obj) => {
                // Map existing v1 licenses to SPDX.
                let mut list = Vec::with_capacity(obj.len());
                for (k, v) in obj.iter() {
                    // These values are derived from actual releases on PGXN.
                    // Inspected by running the following query in its
                    // database:
                    //
                    // `SELECT DISTINCT jsonb(meta)->>'license' FROM distributions;`
                    match (k.as_str(), v.as_str()) {
                        ("PostgreSQL", _) => list.push(k.to_string()),
                        ("Apache", _) => list.push("Apache-2.0".to_string()),
                        ("ISC", _) => list.push(k.to_string()),
                        ("mit", _) => list.push("MIT".to_string()),
                        ("mozilla_2_0", _) => list.push("MPL-2.0".to_string()),
                        ("gpl_3", _) => list.push("GPL-3.0-only".to_string()),
                        ("BSD", _) => list.push("BSD-2-Clause".to_string()),
                        ("BSD 2 Clause", _) => list.push("BSD-2-Clause".to_string()),
                        (
                            "restricted",
                            Some("https://github.com/diffix/pg_diffix/blob/master/LICENSE.md"),
                        ) => list.push("BUSL-1.1".to_string()),
                        _ => return Err(Box::from(format!("Unknown v1 license: {k}: {v}"))),
                    }
                }
                return Ok(Value::String(list.join(" OR ")));
            }
            _ => Err(Box::from(format!("Invalid v1 license: {license}"))),
        };
    }
    Err(Box::from("license property missing"))
}

/// license_expression_for maps the list of v1 open source license names to
/// valid SPDX license. The only ones not currently supported are:
///
/// *   `open_source`
/// *   `restricted`
/// *   `unrestricted`
/// *   `ssleay` (Not used on PGXN v1)
/// *   `unknown`: (Not used on PGXN v1)
fn license_expression_for(name: &str) -> Option<&str> {
    match name {
        "agpl_3" => Some("AGPL-3.0"),
        "apache_1_1" => Some("Apache-1.1"),
        "apache_2_0" => Some("Apache-2.0"),
        "artistic_1" => Some("Artistic-1.0"),
        "artistic_2" => Some("Artistic-2.0"),
        "bsd" => Some("BSD-3-Clause"),
        "freebsd" => Some("BSD-2-Clause-FreeBSD"),
        "gfdl_1_2" => Some("GFDL-1.2-or-later"),
        "gfdl_1_3" => Some("GFDL-1.3-or-later"),
        "gpl_1" => Some("GPL-1.0-only"),
        "gpl_2" => Some("GPL-2.0-only"),
        "gpl_3" => Some("GPL-3.0-only"),
        "lgpl_2_1" => Some("LGPL-2.1"),
        "lgpl_3_0" => Some("LGPL-3.0"),
        "mit" => Some("MIT"),
        "mozilla_1_0" => Some("MPL-1.0"),
        "mozilla_1_1" => Some("MPL-1.1"),
        "openssl" => Some("OpenSSL"),
        "perl_5" => Some("Artistic-1.0-Perl OR GPL-1.0-or-later"),
        "postgresql" => Some("PostgreSQL"),
        "qpl_1_0" => Some("QPL-1.0"),
        "sun" => Some("SISSL"),
        "zlib" => Some("Zlib"),
        _ => None,
    }
}

/// v1_to_v2_contents converts a v1 `provides` object to a v2 `extensions` It
/// sets the `sql` property to the name of the extension + `.control`, which
/// will nearly always be correct. However, some v1 distributions may have the
/// extension in a subdirectory. Others will not be extensions, but modules or
/// apps, in which case the result here will be incorrect, though valid v2
/// metadata.
///
/// Returns the resulting object in a valid `contents` object with
/// `extensions` as the sole property.
fn v1_to_v2_contents(v1: &Value) -> Result<Value, Box<dyn Error>> {
    if let Some(provides) = v1.get("provides") {
        // Assume everything is an extension. It's not true, but most common.
        let mut extensions = Map::new();
        if let Value::Object(obj) = provides {
            for (ext, spec) in obj {
                match spec {
                    Value::Object(obj) => {
                        let mut v2_spec = Map::new();
                        // Assume control file is in the distribution root.
                        v2_spec.insert(
                            "control".to_string(),
                            Value::String(ext.to_string() + ".control"),
                        );

                        // Assume file points to an SQL file (it usually does).
                        if obj.contains_key("file") {
                            v2_spec.insert("sql".to_string(), obj["file"].clone());
                        } else {
                            v2_spec.insert("sql".to_string(), Value::String("UNKNOWN".to_string()));
                        }

                        // Clone directly compatible properties.
                        for (v2, v1) in [("doc", "docfile"), ("abstract", "abstract")] {
                            if obj.contains_key(v1) {
                                v2_spec.insert(v2.to_string(), obj[v1].clone());
                            }
                        }

                        // Copy extension custom properties.
                        v1_to_v2_custom_props(obj, &mut v2_spec);

                        extensions.insert(ext.to_string(), Value::Object(v2_spec));
                    }
                    _ => {
                        return Err(Box::from(format!(
                            "Invalid v1 {:?} extension value: {spec}",
                            ext,
                        )))
                    }
                }
            }
        } else {
            return Err(Box::from(format!("Invalid v1 provides value: {provides}")));
        }

        return Ok(json!({"extensions": extensions}));
    }
    Err(Box::from("provides property missing"))
}

/// v1_to_v2_classifications clones the tags array in v1 into an object with
/// `tags` as the key. Returns None if v2 has no `tags` key.
fn v1_to_v2_classifications(v1: &Value) -> Option<Value> {
    v1.get("tags").map(|tags| json!({"tags": tags.clone()}))
}

/// v1_to_v2_ignore clones the values from the `no_index` key in v1 into a
/// single array for the `ignore` key in a v2 object. All paths under `file`
/// and `directory` are copied into the returned array, with any duplicates
/// removed. Returns None if `no_index` isn't present, isn't an object, if that
///object lacks `file` or `directory` keys, or if no values are found.
fn v1_to_v2_ignore(v1: &Value) -> Option<Value> {
    match v1.get("no_index") {
        Some(Value::Object(ni)) => {
            // Merge the file and directly arrays into a single array.
            let mut ignore: Vec<Value> = Vec::new();
            for k in ["file", "directory"] {
                if let Some(Value::Array(v)) = ni.get(k) {
                    for path in v {
                        if !ignore.contains(path) {
                            ignore.push(path.clone())
                        }
                    }
                }
            }
            if ignore.is_empty() {
                None
            } else {
                Some(Value::Array(ignore))
            }
        }
        _ => None,
    }
}

/// v1_to_v2_dependencies rejiggers v1 `prereqs` metadata into v1
/// `dependencies`. Dependencies on core extensions are specified using
/// `pkg:postgres` purls and all others are specified using `pkg:pgxn` purls.
/// The exception is PostgreSQL dependencies, which are specified under the v1
/// `postgres` key with the lowest version found. v2 does not currently
/// support suggesting or recommending higher versions.
fn v1_to_v2_dependencies(v1: &Value) -> Option<Value> {
    use semver::Version;
    match v1.get("prereqs") {
        Some(Value::Object(prereqs)) => {
            // Track Postgres version requirement. We want lowest version, so
            // compare against unlikely v9999.
            let max_version = Version::parse("9999.0.0").unwrap();
            let mut pg_version = max_version.clone();

            // Iterate over the v2 phases mapped to the v2 phases.
            let mut dependencies = Map::new();
            let mut packages = Map::new();
            for (phase1, phase2) in [
                ("develop", "develop"),
                ("configure", "configure"),
                ("build", "build"),
                ("test", "test"),
                ("runtime", "run"),
            ] {
                if let Some(Value::Object(relation)) = prereqs.get(phase1) {
                    // We have a relation. Iterate through the list of phases.
                    let mut phase = Map::new();
                    for rel_name in ["requires", "recommends", "suggests", "conflicts"] {
                        if let Some(Value::Object(spec)) = relation.get(rel_name) {
                            // We have a phase. Iterate through its list of
                            // key/value pairs.
                            let mut deps = Map::new();
                            for (name, version) in spec {
                                let ext = name.to_lowercase();
                                if ext == "postgresql" {
                                    // Keep the lowest version of Postgres. v1
                                    // doesn't support recommending a higher version.
                                    if let Value::String(version) = version {
                                        if let Ok(pgv) = Version::parse(version) {
                                            if pgv < pg_version {
                                                pg_version = pgv;
                                            }
                                        }
                                    }
                                } else {
                                    // Insert a purl into the deps object.
                                    // Does ext need to be URI-path encoded?
                                    deps.insert(
                                        format!("pkg:{}/{ext}", source_for(&ext)),
                                        version.clone(),
                                    );
                                }
                            }

                            if !deps.is_empty() {
                                phase.insert(rel_name.to_string(), Value::Object(deps));
                            }
                        }
                    }

                    // Copy phase custom properties to the phase.
                    v1_to_v2_custom_props(relation, &mut phase);

                    if !phase.is_empty() {
                        packages.insert(phase2.to_string(), Value::Object(phase));
                    }
                }
            }

            // Set the Postgres version if we have one.
            if pg_version < max_version {
                dependencies.insert("postgres".to_string(), json!({"version": pg_version}));
            }

            // Copy prereqs custom properties to the packages.
            v1_to_v2_custom_props(prereqs, &mut packages);

            // If we have extensions, add them.
            if !packages.is_empty() {
                dependencies.insert("packages".to_string(), Value::Object(packages));
                return Some(Value::Object(dependencies));
            }

            // Return em if we got em.
            match dependencies.is_empty() {
                false => Some(Value::Object(dependencies)),
                true => None,
            }
        }
        _ => None,
    }
}

/// source_for returns the purl source for ext, which must be lowercase.
/// Return "postgres" if ext is a Postgres core extension or PL and "pgxn" for
/// all other values.
fn source_for(ext: &str) -> String {
    match ext {
        "adminpack"
        | "amcheck"
        | "auth_delay"
        | "auto_explain"
        | "basebackup_to_shell"
        | "basic_archive"
        | "bloom"
        | "bool_plperl"
        | "btree_gin"
        | "btree_gist"
        | "chkpass"
        | "citext"
        | "cube"
        | "dblink"
        | "dict_int"
        | "dict_xsyn"
        | "earthdistance"
        | "file_fdw"
        | "fuzzystrmatch"
        | "hstore"
        | "hstore_plperl"
        | "hstore_plpython"
        | "intagg"
        | "intarray"
        | "isn"
        | "jsonb_plperl"
        | "jsonb_plpython"
        | "lo"
        | "ltree"
        | "ltree_plpython"
        | "oid2name"
        | "old_snapshot"
        | "pageinspect"
        | "passwordcheck"
        | "pg_buffercache"
        | "pg_freespacemap"
        | "pg_prewarm"
        | "pg_standby"
        | "pg_stat_statements"
        | "pg_surgery"
        | "pg_trgm"
        | "pg_visibility"
        | "pg_walinspect"
        | "pgcrypto"
        | "pgrowlocks"
        | "pgstattuple"
        | "plperl"
        | "plperlu"
        | "plpgsql"
        | "plpython"
        | "plpythonu"
        | "plpython2u"
        | "plpython3u"
        | "pltcl"
        | "pltclu"
        | "postgres_fdw"
        | "seg"
        | "sepgsql"
        | "spi"
        | "sslinfo"
        | "start-scripts"
        | "tablefunc"
        | "tcn"
        | "test_decoding"
        | "tsearch2"
        | "tsm_system_rows"
        | "tsm_system_time"
        | "unaccent"
        | "uuid-ossp"
        | "vacuumlo"
        | "xml2" => "postgres".to_string(),
        _ => "pgxn".to_string(),
    }
}

/// v1_to_v2_resources copies v1 resources values to compatible v2 resources
/// values:
///
/// *   v1 `homepage` is copied to v2 `homepage`
/// *   v1 `bugtracker.web` or `bugtracker.mailto` are copied to v2 `issues`
/// *   v1 `repository.web` or `repository.url` are copied to v2 `repository`
///
/// Returns `None` if v1 has no compatible resources.
fn v1_to_v2_resources(v1: &Value) -> Option<Value> {
    match v1.get("resources") {
        Some(Value::Object(resources)) => {
            let mut ret = Map::new();
            if let Some(Value::String(home)) = resources.get("homepage") {
                // Directly compatible value.
                ret.insert("homepage".to_string(), json!(home));
            }

            if let Some(Value::Object(bug)) = resources.get("bugtracker") {
                // Prefer the `web` property and fall back on `mailto`.
                if let Some(Value::String(web)) = bug.get("web") {
                    ret.insert("issues".to_string(), json!(web));
                } else if let Some(Value::String(mail)) = bug.get("mailto") {
                    ret.insert("issues".to_string(), json!(format!("mailto:{mail}")));
                }
            }

            if let Some(Value::Object(repo)) = resources.get("repository") {
                // Prefer the `web` property and fall back on `url`.
                if let Some(Value::String(web)) = repo.get("web") {
                    ret.insert("repository".to_string(), json!(web));
                } else if let Some(Value::String(url)) = repo.get("url") {
                    ret.insert("repository".to_string(), json!(url));
                }
            }

            // Copy any custom fields.
            v1_to_v2_custom_props(resources, &mut ret);

            // Return the resources if we have any.
            if ret.is_empty() {
                None
            } else {
                Some(Value::Object(ret))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
