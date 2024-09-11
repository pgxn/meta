use std::{
    env,
    error::Error,
    ffi::OsString,
    fs::File,
    io::{self, Write},
    process::ExitCode,
};

use pgxn_meta::valid::Validator;
use serde_json::Value;

// Minimal main function; logical is all in run.
fn main() -> Result<ExitCode, Box<dyn Error>> {
    run(io::stdout(), env::args_os())
}

// Run the validator. Output will be sent to `out` and options will be parsed
// from `args`.
fn run<I>(mut out: impl Write, args: I) -> Result<ExitCode, Box<dyn Error>>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    let res = parse_args(&mut out, args)?;
    if !res.exit {
        // parse_args() doesn't need to exit, so do the thing.
        validate(&res.file)?;
        writeln!(out, "{} is OK", &res.file)?;
    }

    // If we got here, wer were successful.
    Ok(ExitCode::SUCCESS)
}

// process_args() parses argument into this struct.
struct Args {
    exit: bool,
    file: String,
}

// The default name of the file to validate.
const META_FILE: &str = "META.json";

// Parses the arguments in `args` and returns Args. Output is sent to `out`.
// If `Args.exit` is true, the caller should do no more processing.
fn parse_args<I>(out: &mut impl Write, args: I) -> Result<Args, Box<dyn Error>>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    use lexopt::prelude::*;
    let mut res = Args {
        exit: false,
        file: String::from(META_FILE),
    };
    let mut parser = lexopt::Parser::from_iter(args);

    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => {
                usage(out, &parser)?;
                res.exit = true
            }
            Short('v') | Long("version") => {
                version(out, &parser)?;
                res.exit = true
            }
            Short('m') | Long("man") => {
                docs(out)?;
                res.exit = true
            }
            // Last one wins. Raise an error instead?
            Value(val) => res.file = val.string()?,
            _ => return Err(Box::new(arg.unexpected())),
        }
    }

    Ok(res)
}

// Validates `file`. Panics on validation failure.
fn validate(file: &str) -> Result<(), Box<dyn Error>> {
    match File::open(file) {
        Ok(f) => {
            let meta: Value = serde_json::from_reader(f)?;
            let mut v = Validator::new();
            if let Err(e) = v.validate(&meta) {
                return Err(format!("{file} {e}").into());
            };
            Ok(())
        }
        Err(e) => Err(format!("Cannot open '{file}': {e}").into()),
    }
}

// Returns the binary name from the argument parser and falls back on the name
// determined at compile time.
macro_rules! bn {
    ($x:expr) => {{
        $x.bin_name().unwrap_or(env!("CARGO_BIN_NAME"))
    }};
}

// Outputs a usage statement.
fn usage(out: &mut impl Write, p: &lexopt::Parser) -> Result<(), Box<dyn Error>> {
    writeln!(
        out,
        "Usage: {} [--help | h] [--version | -v] [<path>]\n\n\
        Options:\n\
        \x20 -h --help     Print this usage statement and exit\n\
        \x20 -v --version  Print the version number and exit",
        bn!(p),
    )?;
    Ok(())
}

// Outputs a version statement.
fn version(out: &mut impl Write, p: &lexopt::Parser) -> Result<(), Box<dyn Error>> {
    writeln!(out, "{} {}", bn!(p), env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

// Outputs docs. Or will eventually.
fn docs(out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(out, "Docs")?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
