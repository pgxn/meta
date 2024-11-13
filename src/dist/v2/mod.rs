use super::Distribution;
use crate::error::Error;
use serde_json::Value;

pub fn from_value(meta: Value) -> Result<Distribution, Error> {
    Ok(serde_json::from_value(meta)?)
}
