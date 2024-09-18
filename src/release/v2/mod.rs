use super::Release;
use serde_json::Value;
use std::error::Error;

pub fn from_value(meta: Value) -> Result<Release, Box<dyn Error>> {
    match serde_json::from_value(meta) {
        Ok(m) => Ok(m),
        Err(e) => Err(Box::from(e)),
    }
}
