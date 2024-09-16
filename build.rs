// build.rs

use serde_json::{map::Map, Value};
use std::{env, error::Error, fs::File, io::Write, path::Path};
use wax::Glob;

fn main() -> Result<(), Box<dyn Error>> {
    merge_version(1)?;
    merge_version(2)?;
    println!("cargo::rerun-if-changed=schema");
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}

fn merge_version(version: u8) -> Result<(), Box<dyn Error>> {
    // Open the file to write each schema to.
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dst_file = format!("pgxn-meta-v{version}.schemas.json");
    let dst_path = Path::new(&out_dir).join(&dst_file);
    let file = File::create(dst_path)?;

    // Set up the search directory.
    let src_dir = Path::new("schema").join(format!("v{version}"));
    let glob = Glob::new("*.schema.json")?;

    // Write each file to the destination.
    for path in glob.walk(src_dir) {
        let path = &path?.into_path();
        let schema: Map<String, Value> = serde_json::from_reader(File::open(path)?)?;
        serde_json::to_writer(&file, &schema)?;
        writeln!(&file)?;
    }

    Ok(())
}
