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
pub struct RawLink<'a> {
    pub uri: URI<'a>,
}

impl<'a> From<URI<'a>> for RawLink<'a> {
    fn from(uri: URI<'a>) -> Self {
        Self::new(uri)
    }
}

impl<'a> TryFrom<&'a str> for RawLink<'a> {
    type Error = uriparse::URIError;

    fn try_from(str_uri: &'a str) -> Result<Self, Self::Error> {
        Ok(Self::from(URI::try_from(str_uri)?))
    }
}
