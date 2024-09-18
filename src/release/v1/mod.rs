use super::Release;
use crate::meta::v1 as dist;
use serde_json::{json, Value};
use std::error::Error;

/// to_v2 parses v1, which contains PGXN v1 release metadata, into a JSON
/// object containing valid PGXN v2 release metadata.
pub fn to_v2(v1: &Value) -> Result<Value, Box<dyn Error>> {
    let mut v2_val = dist::to_v2(v1)?;
    let v2 = v2_val
        .as_object_mut()
        .ok_or("Date returned from v1::to_v2 is not an object")?;

    // Convert release.
    v2.insert("release".to_string(), v1_to_v2_release(v1)?);

    Ok(v2_val)
}

/// from_value parses v1, which contains PGXN v1 metadata, into a
/// [`Release`] object containing valid PGXN v2 metadata.
pub fn from_value(v1: Value) -> Result<Release, Box<dyn Error>> {
    to_v2(&v1)?.try_into()
}

/// v1_to_v2_release clones release metadata from v1 to the v2 format. The
/// included signature information will be random and un-verifiable, but
/// adequate for v2 JSON Schema validation.
fn v1_to_v2_release(v1: &Value) -> Result<Value, Box<dyn Error>> {
    use rand::distributions::{Alphanumeric, DistString};
    let mut field = "user";
    if let Some(Value::String(user)) = v1.get(field) {
        field = "date";
        if let Some(Value::String(date)) = v1.get(field) {
            field = "sha1";
            if let Some(Value::String(sha1)) = v1.get(field) {
                field = "name";
                if let Some(Value::String(name)) = v1.get(field) {
                    field = "version";
                    if let Some(Value::String(version)) = v1.get(field) {
                        let uri =
                            format!("dist/{1}/{0}/{1}-{0}.zip", version.as_str(), name.as_str());
                        // Generate random base64-ish values to mock headers
                        // and signatures.
                        let mut rng = rand::thread_rng();
                        return Ok(json!({
                            "headers": [format!("eyJ{}", Alphanumeric.sample_string(&mut rng, 13))],
                            "signatures": [Alphanumeric.sample_string(&mut rng, 32)],
                            "payload": {
                                "user": user,
                                "date": date,
                                "uri": uri,
                                "digests": {"sha1": sha1},
                            }
                        }));
                    }
                }
            }
        }
    }
    Err(Box::from(format!("missing release property {:?}", field)))
}

#[cfg(test)]
mod tests;
