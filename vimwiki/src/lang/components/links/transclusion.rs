use super::Description;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use url::Url;

/// Represents a link that is used as a "Wiki Include" to pull in resources
#[derive(Constructor, Clone, Debug, Eq, Serialize, Deserialize)]
pub struct TransclusionLink {
    pub url: Url,
    pub description: Option<Description>,
    pub properties: HashMap<String, String>,
}

impl TransclusionLink {
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

impl PartialEq for TransclusionLink {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.description == other.description
            && self.properties == other.properties
    }
}

impl Hash for TransclusionLink {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.description.hash(state);

        // Grab all property keys and sort them so we get a reproducible
        // iteration over the keys
        let mut keys = self.properties.keys().collect::<Vec<&String>>();
        keys.sort_unstable();

        // Use property keys in hash
        for k in keys.drain(..) {
            k.hash(state);
        }
    }
}

impl From<Url> for TransclusionLink {
    fn from(url: Url) -> Self {
        Self::new(url, None, HashMap::default())
    }
}

impl TryFrom<&str> for TransclusionLink {
    type Error = url::ParseError;

    fn try_from(str_url: &str) -> Result<Self, Self::Error> {
        Ok(Self::from(Url::parse(str_url)?))
    }
}
