use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use uriparse::URI;

/// Represents a raw link in the form of http[s]://example.com
#[derive(
    Constructor,
    Clone,
    Debug,
    Display,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{}", uri)]
pub struct RawLink {
    pub uri: URI<'static>,
}

impl From<URI<'static>> for RawLink {
    fn from(uri: URI<'static>) -> Self {
        Self::new(uri)
    }
}

impl TryFrom<&str> for RawLink {
    type Error = uriparse::URIError;

    fn try_from(str_uri: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(URI::try_from(str_uri)?.into_owned()))
    }
}
