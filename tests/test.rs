use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
};

use boon::{Compiler, Schemas};
use serde_json::Value;

#[test]
fn test_schema_v1() -> Result<(), Box<dyn Error>> {
    let mut compiler = Compiler::new();
    let mut loaded: HashMap<String, Vec<Value>> = HashMap::new();

    let paths = fs::read_dir("./schema/v1")?;
    for path in paths {
        let path = path?.path();
        let bn = path.file_name().unwrap().to_str().unwrap();
        if bn.ends_with(".schema.json") {
            let schema: Value = serde_json::from_reader(File::open(path.clone())?)?;
            if let Value::String(s) = &schema["$id"] {
                // Make sure that the ID is correct.
                assert_eq!(format!("https://pgxn.org/meta/v1/{bn}"), *s);

                // Add the schema to the compiler.
                compiler.add_resource(s, schema.to_owned())?;

                // Grab the examples, if any, to test later.
                if let Value::Array(a) = &schema["examples"] {
                    loaded.insert(s.clone(), a.to_owned());
                } else {
                    loaded.insert(s.clone(), Vec::new());
                }
            } else {
                panic!("Unable to find ID in {}", path.display());
            }
        }
    }

    // Make sure we found schemas.
    assert!(!loaded.is_empty());

    // Make sure each schema we loaded is valid.
    let mut schemas = Schemas::new();
    for (id, examples) in loaded {
        let index = compiler.compile(id.as_str(), &mut schemas)?;
        println!("{} ok", id);

        // Test the schema's examples.
        for (i, example) in examples.iter().enumerate() {
            if let Err(e) = schemas.validate(example, index) {
                panic!("Example {i} failed: {e}");
            }
            println!("  Example {i} ok");
        }
    }

    Ok(())
}
