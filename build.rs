// build.rs

use serde_json::{map::Map, Value};
use std::{env, error::Error, fs::File, path::Path};
use wax::Glob;

fn main() -> Result<(), Box<dyn Error>> {
    merge_version(1)?;
    merge_version(2)?;
    println!("cargo::rerun-if-changed=schema");
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}

fn merge_version(version: u8) -> Result<(), Box<dyn Error>> {
    let src_dir = Path::new("schema").join(format!("v{version}"));
    let glob = Glob::new("*.schema.json")?;
    let mut defs = serde_json::map::Map::new();

    for path in glob.walk(src_dir) {
        let path = &path?.into_path();
        let schema: Map<String, Value> = serde_json::from_reader(File::open(path)?)?;
        // let id = schema["$id"].as_str().ok_or("No $id found in {path}")?;
        let id = path.file_name().unwrap().to_str().unwrap();
        defs.insert(id.to_string(), Value::Object(schema));
    }

    const ROOT: &str = "distribution.schema.json";
    let mut dist = defs.remove(ROOT).unwrap();
    dist.as_object_mut()
        .unwrap()
        .insert("$defs".to_string(), Value::Object(defs));

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dst_file = format!("pgxn-meta-v{version}.schema.json");
    let dst_path = Path::new(&out_dir).join(&dst_file);
    let file = File::create(dst_path)?;
    serde_json::to_writer(&file, &dist)?;

    Ok(())
}
