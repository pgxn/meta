use super::Release;
use crate::error::Error;
use serde_json::Value;

pub fn from_value(meta: Value) -> Result<Release, Error> {
    Ok(serde_json::from_value(meta)?)
}
