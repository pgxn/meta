use super::*;
use std::{fs::File, path::PathBuf};

#[test]
fn test_v1_v2_common() {
    let meta = json!({
      "version": "2.0.0",
      "url": "https://rfcs.pgxn.org/0003-meta-spec-v2.html"
    });

    for (name, input, expect) in [
        (
            "basics",
            json!({"name": "pair", "version": "1.2.3", "abstract": "this and that", "description": "Howdy", "x_foo": true}),
            json!({"name": "pair", "version": "1.2.3", "abstract": "this and that", "description": "Howdy", "x_foo": true, "meta-spec": meta}),
        ),
        (
            "name only",
            json!({"name": "pair"}),
            json!({"name": "pair", "meta-spec": meta}),
        ),
        (
            "name and version",
            json!({"name": "pair", "version": "1.2.3"}),
            json!({"name": "pair", "version": "1.2.3", "meta-spec": meta}),
        ),
        (
            "meta spec custom props",
            json!({"meta-spec": {"x_foo": 1, "X_y": true}}),
            json!({"meta-spec": {
              "version": "2.0.0",
              "url": "https://rfcs.pgxn.org/0003-meta-spec-v2.html",
              "x_foo": 1,
              "X_y": true
            }}),
        ),
        (
            "renamed fields",
            json!({"generated_by": "meta_spec v0.2.0"}),
            json!({"producer": "meta_spec v0.2.0", "meta-spec": meta}),
        ),
        (
            "name and version",
            json!({"name": "pair", "version": "1.2.3"}),
            json!({"name": "pair", "version": "1.2.3", "meta-spec": meta}),
        ),
    ] {
        let v2 = v1_to_v2_common(&input);
        assert_eq!(expect, Value::Object(v2), "{name}");
    }
}

#[test]
fn test_v1_to_v2_maintainers() {
    for (name, input, expect) in [
        (
            "name_and_email",
            json!({"maintainer": "Barrack Obama <potus@example.com>"}),
            json!([{"name": "Barrack Obama", "email": "potus@example.com"}]),
        ),
        (
            "name_only",
            json!({"maintainer": "Barrack Obama"}),
            json!([{"name": "Barrack Obama", "url": "https://pgxn.org"}]),
        ),
        (
            "name_and_homepage",
            json!({"maintainer": "Barrack Obama", "resources": {"homepage": "https://example.com"}}),
            json!([{"name": "Barrack Obama", "url": "https://example.com"}]),
        ),
        (
            "name_and_invalid_homepage",
            json!({"maintainer": "Barrack Obama", "resources": {"homepage": 42}}),
            json!([{"name": "Barrack Obama", "url": "https://pgxn.org"}]),
        ),
        (
            "email_only",
            json!({"maintainer": "potus@example.com"}),
            json!([{"name": "potus@example.com", "email": "potus@example.com"}]),
        ),
        (
            "two_maintainers",
            json!({"maintainer": [
                "David E. Wheeler <theory@pgxn.org>",
                "Josh Berkus <jberkus@pgxn.org>"
            ]}),
            json!([
                {"name": "David E. Wheeler", "email": "theory@pgxn.org"},
                {"name": "Josh Berkus", "email": "jberkus@pgxn.org"},
            ]),
        ),
        (
            "varying maintainer_formats",
            json!({"maintainer": ["David E. Wheeler <theory@pgxn.org>", "Josh Berkus, Esq."]}),
            json!([
                {"name": "David E. Wheeler", "email": "theory@pgxn.org"},
                {"name": "Josh Berkus, Esq.", "url": "https://pgxn.org"},
            ]),
        ),
    ] {
        match v1_to_v2_maintainers(&input) {
            Ok(maintainers) => assert_eq!(expect, maintainers, "{name}"),
            Err(e) => panic!("{name}: {e}"),
        }
    }
}

#[test]
fn test_v1_to_v2_maintainers_errors() {
    for (name, input, err) in [
        (
            "no maintainer",
            json!({"name": "pair", "version": "1.2.4"}),
            "maintainer property missing",
        ),
        (
            "null maintainer",
            json!({"maintainer": null}),
            "Invalid v1 maintainer: null",
        ),
        (
            "object maintainer",
            json!({"maintainer": {"name": "hi"}}),
            r#"Invalid v1 maintainer: {"name":"hi"}"#,
        ),
        (
            "null maintainer item",
            json!({"maintainer": [null]}),
            "Invalid v1 maintainer: null",
        ),
        (
            "true maintainer item",
            json!({"maintainer": [true]}),
            "Invalid v1 maintainer: true",
        ),
    ] {
        match v1_to_v2_maintainers(&input) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }
}

#[test]
fn test_license_expression_for() {
    for (v1_name, v2_name) in [
        ("agpl_3", "AGPL-3.0"),
        ("apache_1_1", "Apache-1.1"),
        ("apache_2_0", "Apache-2.0"),
        ("artistic_1", "Artistic-1.0"),
        ("artistic_2", "Artistic-2.0"),
        ("bsd", "BSD-3-Clause"),
        ("freebsd", "BSD-2-Clause-FreeBSD"),
        ("gfdl_1_2", "GFDL-1.2-or-later"),
        ("gfdl_1_3", "GFDL-1.3-or-later"),
        ("gpl_1", "GPL-1.0-only"),
        ("gpl_2", "GPL-2.0-only"),
        ("gpl_3", "GPL-3.0-only"),
        ("lgpl_2_1", "LGPL-2.1"),
        ("lgpl_3_0", "LGPL-3.0"),
        ("mit", "MIT"),
        ("mozilla_1_0", "MPL-1.0"),
        ("mozilla_1_1", "MPL-1.1"),
        ("openssl", "OpenSSL"),
        ("perl_5", "Artistic-1.0-Perl OR GPL-1.0-or-later"),
        ("postgresql", "PostgreSQL"),
        ("qpl_1_0", "QPL-1.0"),
        ("sun", "SISSL"),
        ("zlib", "Zlib"),
    ] {
        let v2 = license_expression_for(v1_name);
        assert_eq!(Some(v2_name), v2);
    }

    // V1 License not included in the SPDX license list.
    for v1_name in [
        "ssleay",
        "open_source",
        "restricted",
        "unrestricted",
        "unknown",
    ] {
        assert_eq!(None, license_expression_for(v1_name));
    }
}

#[test]
fn test_v1_v2_licenses() {
    for (name, input, expect) in [
        (
            "apache_2_0",
            json!({"license": "apache_2_0"}),
            json!("Apache-2.0"),
        ),
        (
            "perl_5",
            json!({"license": "perl_5"}),
            json!("Artistic-1.0-Perl OR GPL-1.0-or-later"),
        ),
        (
            "array",
            json!({"license": ["apache_2_0", "perl_5"]}),
            json!("Apache-2.0 OR Artistic-1.0-Perl OR GPL-1.0-or-later"),
        ),
        (
            "another",
            json!({"license": ["mit", "postgresql"]}),
            json!("MIT OR PostgreSQL"),
        ),
        (
            "object",
            json!({"license": {"PostgreSQL": "https://www.postgresql.org/about/licence"}}),
            json!("PostgreSQL"),
        ),
        (
            "object_multiple",
            json!({"license": {
                "PostgreSQL": "https://www.postgresql.org/about/licence",
                "Apache": "http://www.apache.org/licenses/LICENSE-2.0",
            }}),
            json!("Apache-2.0 OR PostgreSQL"),
        ),
        (
            "object_all_others",
            json!({"license": {
                "restricted": "https://github.com/diffix/pg_diffix/blob/master/LICENSE.md",
                "ISC": "http://www.opensource.org/licenses/ISC",
                "mit": "http://en.wikipedia.org/wiki/MIT_License",
                "mozilla_2_0": "https://www.mozilla.org/en-US/MPL/2.0/",
                "gpl_3": "https://www.gnu.org/licenses/gpl-3.0.en.html",
                "BSD 2 Clause": "http://opensource.org/licenses/bsd-license.php",
                "BSD": "http://www.opensource.org/licenses/bsd-license.html"
            }}),
            json!(
                "BSD-2-Clause OR BSD-2-Clause OR ISC OR GPL-3.0-only OR MIT OR MPL-2.0 OR BUSL-1.1"
            ),
        ),
    ] {
        match v1_to_v2_license(&input) {
            Ok(lic) => assert_eq!(expect, lic, "{name}"),
            Err(e) => panic!("{name}: {e}"),
        }
    }
}

#[test]
fn test_v1_v2_licenses_error() {
    for (name, input, err) in [
        (
            "unknown string",
            json!({"license": "nonesuch"}),
            "Invalid v1 license: \"nonesuch\"",
        ),
        (
            "unknown array",
            json!({"license": ["nonesuch"]}),
            "Invalid v1 license: \"nonesuch\"",
        ),
        (
            "array non-string item",
            json!({"license": [{"x": "y"}]}),
            "Invalid v1 license: {\"x\":\"y\"}",
        ),
        (
            "unknown object",
            json!({"license": {"nonesuch": "http://example.com"}}),
            "Unknown v1 license: nonesuch: \"http://example.com\"",
        ),
        ("number", json!({"license": 42}), "Invalid v1 license: 42"),
        ("null", json!({"license": null}), "Invalid v1 license: null"),
        ("nonexistent", json!({}), "license property missing"),
    ] {
        match v1_to_v2_license(&input) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }
}

#[test]
fn test_v1_v2_contents() {
    for (name, input, expect) in [
        (
            "simple",
            json!({"widget": {
                "file": "sql/widget.sql.in",
                "version": "0.2.5",
            }}),
            json!({"widget": {
                "control": "widget.control",
                "sql": "sql/widget.sql.in",
            }}),
        ),
        (
            "full",
            json!({"pair": {
                "abstract": "A key/value pair data type",
                "file": "sql/pair.sql",
                "docfile": "doc/pair.md",
                "version": "0.1.0",
                "x_foo": "hi",
            }}),
            json!({"pair": {
                "control": "pair.control",
                "sql": "sql/pair.sql",
                "abstract": "A key/value pair data type",
                "doc": "doc/pair.md",
                "x_foo": "hi",
            }}),
        ),
        (
            "no file",
            json!({"thing": {}}),
            json!({"thing": {
                "control": "thing.control",
                "sql": "UNKNOWN",
            }}),
        ),
    ] {
        let input = json!({"provides": input});
        let expect = json!({"extensions": expect});
        match v1_to_v2_contents(&input) {
            Ok(ext) => assert_eq!(expect, ext, "{name}"),
            Err(e) => panic!("{name}: {e}"),
        }
    }
}

#[test]
fn test_v1_v2_contents_err() {
    for (name, input, err) in [
        ("no provides", json!({}), "provides property missing"),
        (
            "provides null",
            json!({"provides": null}),
            "Invalid v1 provides value: null",
        ),
        (
            "extension not object",
            json!({"provides": {"foo": []}}),
            "Invalid v1 \"foo\" extension value: []",
        ),
    ] {
        match v1_to_v2_contents(&input) {
            Ok(_) => panic!("{name} unexpectedly succeeded"),
            Err(e) => assert_eq!(err, e.to_string(), "{name}"),
        }
    }
}

#[test]
fn test_v1_v2_classifications() {
    for (name, input, expect) in [
        (
            "one tag",
            json!({"tags": ["xxx"]}),
            Some(json!({"tags": ["xxx"]})),
        ),
        (
            "tags",
            json!({"tags": ["xxx", "yyy"], "name": "pair"}),
            Some(json!({"tags": ["xxx", "yyy"]})),
        ),
        ("no tags", json!({"name": "pair"}), None),
        (
            "null tags",
            json!({"tags": null, "name": "pair"}),
            Some(json!({"tags": null})),
        ),
    ] {
        assert_eq!(expect, v1_to_v2_classifications(&input), "{name}")
    }
}

#[test]
fn test_v1_v2_ignore() {
    for (name, input, expect) in [
        (
            "file",
            json!({"no_index": {"file": ["xxx"]}}),
            Some(json!(["xxx"])),
        ),
        (
            "two files",
            json!({"no_index": {"file": ["xxx", "yyy"]}}),
            Some(json!(["xxx", "yyy"])),
        ),
        (
            "files and directories",
            json!({"no_index": {"file": ["xxx", "yyy"], "directory": ["src/private"]}}),
            Some(json!(["xxx", "yyy", "src/private"])),
        ),
        (
            "dedup",
            json!({"no_index": {"file": ["xxx", "yyy"], "directory": ["src/private", "xxx"]}}),
            Some(json!(["xxx", "yyy", "src/private"])),
        ),
        (
            "ignore other keys",
            json!({"no_index": {"file": ["xxx"], "lol": ["its me"]}}),
            Some(json!(["xxx"])),
        ),
        ("no no_index", json!({"name": "pair"}), None),
        ("null no_index", json!({"no_index": null}), None),
        ("string no_index", json!({"no_index": "hi"}), None),
        ("object no_index", json!({"no_index": {}}), None),
        (
            "empty values",
            json!({"no_index": {"file": [], "directory": []}}),
            None,
        ),
    ] {
        assert_eq!(expect, v1_to_v2_ignore(&input), "{name}")
    }
}

#[test]
fn test_v1_v2_dependencies() {
    for (name, input, expect) in [
        (
            "runtime requires",
            json!({"prereqs": {"runtime": {"requires": {"v8": "1.2.0"}}, "x_foo": 1}}),
            Some(json!({"packages": {"run": {"requires": {"pkg:pgxn/v8": "1.2.0"}}, "x_foo": 1}})),
        ),
        (
            "build core and pgxn",
            json!({"prereqs": {"build": {"requires": {"pgtap": "1.3.0", "plpgsql": "12.0.0"}}}}),
            Some(json!({"packages": {"build": {"requires": {
                    "pkg:pgxn/pgtap": "1.3.0",
                    "pkg:postgres/plpgsql": "12.0.0",
            }}}})),
        ),
        (
            "postgres requires",
            json!({"prereqs": {"runtime": {"requires": {"PostgreSQL": "12.0.0"}}}}),
            Some(json!({"postgres": {"version": "12.0.0"}})),
        ),
        (
            "develop conflicts",
            json!({"prereqs": {"develop": {"conflicts": {"v8": "1.2.0"}}}}),
            Some(json!({"packages": {"develop": {"conflicts": {"pkg:pgxn/v8": "1.2.0"}}}})),
        ),
        (
            "lots of stuff",
            json!({"prereqs": {
                "runtime": {
                    "requires": {
                        "PostgreSQL": "8.0.0",
                        "PostGIS": "1.5.0"
                    },
                    "recommends": {
                        "PostgreSQL": "8.4.0"
                    },
                    "suggests": {
                        "sha1": 0
                    }
                },
                "build": {
                    "requires": {
                        "prefix": 0
                    }
                },
                "test": {
                    "recommends": {
                        "pgTAP": 0
                    }
                },
                "x_go_time": "now",
            }}),
            Some(json!({
                "postgres": {"version": "8.0.0"},
                "packages": {
                    "run": {
                        "requires": {"pkg:pgxn/postgis": "1.5.0"},
                        "suggests": {"pkg:pgxn/sha1": 0},
                    },
                    "build": {
                        "requires": {"pkg:pgxn/prefix": 0},
                    },
                    "test": {
                        "recommends": {"pkg:pgxn/pgtap": 0},
                    },
                    "x_go_time": "now",
                }
            })),
        ),
        (
            "invalid postgres requires",
            json!({"prereqs": {"runtime": {"requires": {"PostgreSQL": "nope"}}}}),
            None,
        ),
        (
            "null postgres requires",
            json!({"prereqs": {"runtime": {"requires": {"PostgreSQL": null}}}}),
            None,
        ),
        (
            "unknown phase",
            json!({"prereqs": {"lol": {"requires": {"isn": 0}}}}),
            None,
        ),
        ("null prereqs", json!({"prereqs": null}), None),
        ("array prereqs", json!({"prereqs": []}), None),
        ("number prereqs", json!({"prereqs": 42}), None),
        ("string prereqs", json!({"prereqs": "hi"}), None),
    ] {
        assert_eq!(expect, v1_to_v2_dependencies(&input), "{name}")
    }
}

#[test]
fn test_source_for() {
    for name in [
        "adminpack",
        "amcheck",
        "auth_delay",
        "auto_explain",
        "basebackup_to_shell",
        "basic_archive",
        "bloom",
        "bool_plperl",
        "btree_gin",
        "btree_gist",
        "chkpass",
        "citext",
        "cube",
        "dblink",
        "dict_int",
        "dict_xsyn",
        "earthdistance",
        "file_fdw",
        "fuzzystrmatch",
        "hstore",
        "hstore_plperl",
        "hstore_plpython",
        "intagg",
        "intarray",
        "isn",
        "jsonb_plperl",
        "jsonb_plpython",
        "lo",
        "ltree",
        "ltree_plpython",
        "oid2name",
        "old_snapshot",
        "pageinspect",
        "passwordcheck",
        "pg_buffercache",
        "pg_freespacemap",
        "pg_prewarm",
        "pg_standby",
        "pg_stat_statements",
        "pg_surgery",
        "pg_trgm",
        "pg_visibility",
        "pg_walinspect",
        "pgcrypto",
        "pgrowlocks",
        "pgstattuple",
        "plperl",
        "plperlu",
        "plpgsql",
        "plpython",
        "plpythonu",
        "plpython2u",
        "plpython3u",
        "pltcl",
        "pltclu",
        "postgres_fdw",
        "seg",
        "sepgsql",
        "spi",
        "sslinfo",
        "start-scripts",
        "tablefunc",
        "tcn",
        "test_decoding",
        "tsearch2",
        "tsm_system_rows",
        "tsm_system_time",
        "unaccent",
        "uuid-ossp",
        "vacuumlo",
        "xml2",
    ] {
        assert_eq!("postgres".to_string(), source_for(name), "{name}")
    }

    // SELECT DISTINCT jsonb_object_keys(jsonb_path_query(jsonb(meta), '$.prereqs.*.*')) FROM distributions;
    for name in [
        "berkeleydb",
        "v8",
        "lambda",
        "vectorscale",
        "pg_partman",
        "ddlx",
        "pgtap",
        "pgtap",
        "pgvector",
        "trunklet",
        "variant",
        "pg_jobmon",
        "pg_cron",
        "columnar",
        "bignum",
        "oracle",
        "pgmq",
        "cat_tools",
        "plproxy",
        "pg_readme",
        "postgis",
        "python",
        "pgbitmap",
        "pg_utility_trigger_functions",
    ] {
        assert_eq!("pgxn".to_string(), source_for(name), "{name}")
    }
}

#[test]
fn test_v1_v2_resources() {
    for (name, input, expect) in [
        ("no resources", json!({"name": "hi"}), None),
        ("empty resources", json!({"resources": {}}), None),
        ("null resources", json!({"resources": null}), None),
        ("string resources", json!({"resources": "hi"}), None),
        ("number resources", json!({"resources": 42}), None),
        ("bool resources", json!({"resources": true}), None),
        (
            "homepage",
            json!({"name": "hi", "resources": {"homepage": "https://pgtap.org"}}),
            Some(json!({"homepage": "https://pgtap.org"})),
        ),
        (
            "bugtracker web",
            json!({"resources": {"bugtracker": {
                "web": "https://github.org/pgxn/meta/issues",
                "mailto": "hi@example.com",
            }}}),
            Some(json!({"issues": "https://github.org/pgxn/meta/issues"})),
        ),
        (
            "bugtracker mailto",
            json!({"resources": {"bugtracker": {
                "mailto": "hi@example.com",
            }}}),
            Some(json!({"issues": "mailto:hi@example.com"})),
        ),
        (
            "repo web",
            json!({"resources": {"repository": {
                "web": "https://github.org/pgxn/meta",
                "url": "git@github.com:pgxn/meta.git",
            }}}),
            Some(json!({"repository": "https://github.org/pgxn/meta"})),
        ),
        (
            "repo url",
            json!({"resources": {"repository": {
                "url": "git@github.com:pgxn/meta.git",
            }}}),
            Some(json!({"repository": "git@github.com:pgxn/meta.git"})),
        ),
        (
            "repo url plus custom",
            json!({"resources": {
                "repository": { "url": "git@github.com:pgxn/meta.git", },
                "x_foo": true,
            }}),
            Some(json!({
                "repository": "git@github.com:pgxn/meta.git",
                "x_foo": true,
            })),
        ),
    ] {
        assert_eq!(expect, v1_to_v2_resources(&input), "{name}")
    }
}

#[test]
fn test_from_value() -> Result<(), Box<dyn Error>> {
    use wax::Glob;
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "corpus", "v1"]
        .iter()
        .collect();
    let glob = Glob::new("*.json")?;

    for path in glob.walk(dir) {
        let path = path?.into_path();
        let meta: Value = serde_json::from_reader(File::open(&path)?)?;
        if let Err(e) = from_value(meta) {
            panic!("{:?} failed: {e}", path.file_name().unwrap());
        }
        println!("Example {:?} ok", path.file_name().unwrap());
    }

    Ok(())
}
