use super::Distribution;
use serde_json::Value;
use std::error::Error;

pub fn from_value(meta: Value) -> Result<Distribution, Box<dyn Error>> {
    match serde_json::from_value(meta) {
        Ok(m) => Ok(m),
        Err(e) => Err(Box::from(e)),
    }
}
