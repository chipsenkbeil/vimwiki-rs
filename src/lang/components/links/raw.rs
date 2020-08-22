use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use url::Url;

/// Represents a raw link in the form of http[s]://example.com
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct RawLink {
    pub url: Url,
}

impl From<Url> for RawLink {
    fn from(url: Url) -> Self {
        Self::new(url)
    }
}

impl TryFrom<&str> for RawLink {
    type Error = url::ParseError;

    fn try_from(str_url: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(Url::parse(str_url)?))
    }
}
