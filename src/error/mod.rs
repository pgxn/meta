//! PGXN Meta Errors.

#[cfg(test)]
mod tests;

/// Build errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// License Error.
    #[error("{}", .0.reason)]
    License(#[from] spdx::error::ParseError),

    /// Validator cannot determine the version of the meta spec.
    #[error("cannot determine meta-spec version")]
    UnknownSpec,

    /// A schema file has no `$id` property.
    #[error("no $id found in schema")]
    UnknownSchemaId,

    /// JSON Schema compile error.
    #[error(transparent)]
    #[allow(clippy::enum_variant_names)]
    CompileError(Box<boon::CompileError>),

    /// JSON Schema validation error.
    #[error("{0}")]
    #[allow(clippy::enum_variant_names)]
    ValidationError(String),

    /// Serde JSON error.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Glob build error.
    #[error(transparent)]
    Glob(#[from] wax::GlobError),

    /// Parameter errors.
    #[error("{0}")]
    Param(&'static str),

    /// Invalid property value.
    #[error("invalid v{1} {0} value: {2}")]
    Invalid(&'static str, u8, serde_json::Value),

    /// Missing property value.
    #[error("{0} property missing")]
    Missing(&'static str),

    /// Hash digest mismatch.
    #[error("{0} digest {1} does not match {2}")]
    Digest(&'static str, String, String),
}

impl<'s, 'v> From<boon::ValidationError<'s, 'v>> for Error {
    fn from(value: boon::ValidationError<'s, 'v>) -> Self {
        Self::ValidationError(value.to_string())
    }
}

// Box compiler errors.
impl From<boon::CompileError> for Error {
    fn from(value: boon::CompileError) -> Self {
        Self::CompileError(Box::new(value))
    }
}

impl From<wax::BuildError> for Error {
    fn from(value: wax::BuildError) -> Self {
        wax::GlobError::Build(value).into()
    }
}

impl From<wax::WalkError> for Error {
    fn from(value: wax::WalkError) -> Self {
        wax::GlobError::Walk(value).into()
    }
}
