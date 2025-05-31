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
#[path = "tests.rs"]
mod tests;
