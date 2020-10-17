use super::{uri_to_borrowed, Description};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
};
use uriparse::URI;

/// Represents a link that is used as a "Wiki Include" to pull in resources
#[derive(Constructor, Clone, Debug, Eq, Serialize, Deserialize)]
pub struct TransclusionLink<'a> {
    pub uri: URI<'a>,
    pub description: Option<Description<'a>>,
    pub properties: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl TransclusionLink<'_> {
    pub fn to_borrowed(&self) -> TransclusionLink {
        use self::Cow::*;

        let uri = uri_to_borrowed(&self.uri);
        let description =
            self.description.as_ref().map(Description::to_borrowed);
        let properties = self
            .properties
            .iter()
            .map(|(key, value)| {
                let key = Cow::Borrowed(match key {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                });
                let value = Cow::Borrowed(match value {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                });

                (key, value)
            })
            .collect();

        TransclusionLink {
            uri,
            description,
            properties,
        }
    }

    pub fn into_owned(self) -> TransclusionLink<'static> {
        let uri = self.uri.into_owned();
        let description = self.description.map(Description::into_owned);
        let properties = self
            .properties
            .into_iter()
            .map(|(key, value)| {
                (Cow::from(key.into_owned()), Cow::from(value.into_owned()))
            })
            .collect();

        TransclusionLink {
            uri,
            description,
            properties,
        }
    }
}

impl<'a> TransclusionLink<'a> {
    /// Whether or not the associated URL is local to the current system
    pub fn is_local(&self) -> bool {
        let scheme = self.uri.scheme().as_str();
        scheme == "file" || scheme == "local" || scheme.is_empty()
    }

    /// Whether or not the associated URL is targeting a remote system
    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }
}

impl<'a> PartialEq for TransclusionLink<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
            && self.description == other.description
            && self.properties == other.properties
    }
}

impl<'a> Hash for TransclusionLink<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
        self.description.hash(state);

        // Grab all property keys and sort them so we get a reproducible
        // iteration over the keys
        let mut keys = self.properties.keys().collect::<Vec<&Cow<'_, str>>>();
        keys.sort_unstable();

        // Use property keys in hash
        for k in keys.drain(..) {
            k.hash(state);
        }
    }
}

impl<'a> fmt::Display for TransclusionLink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = self.description.as_ref() {
            write!(f, "{}", desc)
        } else {
            write!(f, "{}", self.uri)
        }
    }
}

impl<'a> From<URI<'a>> for TransclusionLink<'a> {
    fn from(uri: URI<'a>) -> Self {
        Self::new(uri, None, HashMap::default())
    }
}

impl<'a> TryFrom<&'a str> for TransclusionLink<'a> {
    type Error = uriparse::URIError;

    fn try_from(str_uri: &'a str) -> Result<Self, Self::Error> {
        Ok(Self::from(URI::try_from(str_uri)?))
    }
}
