use super::*;
use core::panic;
use std::{ffi::OsStr, path::Path, str};

struct TC<'a> {
    name: &'a str,
    args: &'a [&'a str],
    exit: bool,
    file: &'a str,
    out: &'a str,
}

#[test]
fn test_parse_args() -> Result<(), Box<dyn Error>> {
    for tc in [
        TC {
            name: "no args",
            args: &["meta"],
            exit: false,
            file: META_FILE,
            out: "",
        },
        TC {
            name: "short help",
            args: &["meta", "-h"],
            exit: true,
            file: META_FILE,
            out: "Usage: meta [--help | h] [--version | -v] [<path>]\n\n\
                    Options:\n\
                    \x20 -h --help     Print this usage statement and exit\n\
                    \x20 -v --version  Print the version number and exit\n",
        },
        TC {
            name: "long help",
            args: &["meta", "--help"],
            exit: true,
            file: META_FILE,
            out: "Usage: meta [--help | h] [--version | -v] [<path>]\n\n\
                    Options:\n\
                    \x20 -h --help     Print this usage statement and exit\n\
                    \x20 -v --version  Print the version number and exit\n",
        },
        TC {
            name: "short version",
            args: &["meta", "-v"],
            exit: true,
            file: META_FILE,
            out: concat!("meta ", env!("CARGO_PKG_VERSION"), "\n"),
        },
        TC {
            name: "long version",
            args: &["meta", "--version"],
            exit: true,
            file: META_FILE,
            out: concat!("meta ", env!("CARGO_PKG_VERSION"), "\n"),
        },
        TC {
            name: "short man",
            args: &["meta", "-m"],
            exit: true,
            file: META_FILE,
            out: "Docs\n",
        },
        TC {
            name: "long man",
            args: &["meta", "--man"],
            exit: true,
            file: META_FILE,
            out: "Docs\n",
        },
        TC {
            name: "file name",
            args: &["meta", "hello.json"],
            exit: false,
            file: "hello.json",
            out: "",
        },
        TC {
            name: "multiple values",
            args: &["meta", "hello.json", "hi.json"],
            exit: false,
            file: "hi.json",
            out: "",
        },
    ] {
        let mut file: Vec<u8> = Vec::new();
        match parse_args(&mut file, tc.args) {
            Err(e) => panic!("test {} failed: {e}", tc.name),
            Ok(res) => {
                assert_eq!(res.exit, tc.exit);
                assert_eq!(res.file, tc.file);
                assert_eq!(str::from_utf8(&file)?, tc.out);
            }
        }
    }

    // Make sure we get an error for an unknown option.
    let mut file: Vec<u8> = Vec::new();
    match parse_args(&mut file, ["hi", "-x"]) {
        Ok(_) => panic!("Should have failed on -x but did not"),
        Err(e) => {
            assert_eq!(e.to_string(), "invalid option '-x'");
        }
    }

    Ok(())
}

#[test]
fn test_run() -> Result<(), Box<dyn Error>> {
    let meta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("v2")
        .join("minimal.json");
    let ok_output = format!("{} is OK\n", meta.display());

    struct TC<'a> {
        name: &'a str,
        args: &'a [&'a OsStr],
        out: &'a str,
        // code: u8,
    }

    for tc in [
        TC {
            name: "version",
            args: &[OsStr::new("xyz"), OsStr::new("-v")],
            out: concat!("xyz ", env!("CARGO_PKG_VERSION"), "\n"),
        },
        TC {
            name: "pass file",
            args: &[OsStr::new("xyz"), meta.as_os_str()],
            out: &ok_output,
        },
    ] {
        let mut file: Vec<u8> = Vec::new();
        match run(&mut file, tc.args) {
            Err(e) => panic!("test {:} failed: {e}", tc.name),
            Ok(_) => {
                assert_eq!(str::from_utf8(&file)?, tc.out);
            }
        }
    }

    Ok(())
}

#[test]
fn test_validate() -> Result<(), Box<dyn Error>> {
    // Success first.
    let meta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("v2")
        .join("minimal.json");

    match validate(meta.as_os_str().to_str().unwrap()) {
        Ok(_) => (),
        Err(e) => panic!("Validation failed: {e}"),
    }

    // Invalid next.
    let meta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("invalid.json");
    match validate(meta.as_os_str().to_str().unwrap()) {
        Ok(_) => panic!("Should have failed on invalid.json but did not"),
        Err(e) => assert!(e.to_string().contains(" missing properties 'version")),
    }

    // Nonexistent file
    match validate("nonesuch.txt") {
        Ok(_) => panic!("Should have failed unknown file"),
        Err(e) => assert!(e.to_string().starts_with("Cannot open 'nonesuch.txt': ")),
    }

    Ok(())
}

#[test]
fn test_main() {
    match main() {
        Ok(_) => panic!("Should have failed on main() but did not"),
        Err(e) => {
            assert!(e.to_string().starts_with("Cannot open "));
        }
    }
    assert!(main().is_err());
}
