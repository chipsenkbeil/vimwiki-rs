use super::WithDescription;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use url::Url;

/// Represents a link that is used as a "Wiki Include" to pull in resources
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct TransclusionLink {
    url: Url,
    description: Option<String>,
}

impl TransclusionLink {
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Whether or not the associated URL is local to the current system
    pub fn is_local(&self) -> bool {
        let scheme = self.url.scheme();
        scheme == "file" || scheme == "local" || scheme.is_empty()
    }

    /// Whether or not the associated URL is targeting a remote system
    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }
}

impl From<Url> for TransclusionLink {
    fn from(url: Url) -> Self {
        Self::new(url, None)
    }
}

impl TryFrom<&str> for TransclusionLink {
    type Error = url::ParseError;

    fn try_from(str_url: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(Url::parse(str_url)?))
    }
}

impl WithDescription for TransclusionLink {
    fn with_description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
        self
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}
