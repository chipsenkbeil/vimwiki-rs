use super::{Anchor, Description};
use crate::StrictEq;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use uriparse::{Fragment, Scheme, URIReference, URIReferenceError};

/// Represents data for a link to some content, described through a combination
/// of a URI reference and some arbitrary description
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LinkData<'a> {
    uri_ref: URIReference<'a>,
    description: Option<Description<'a>>,
    properties: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
}

impl LinkData<'_> {
    pub fn to_borrowed(&self) -> LinkData {
        use self::Cow::*;

        let uri_ref = uri_ref_to_borrowed(&self.uri_ref);
        let description =
            self.description.as_ref().map(Description::to_borrowed);
        let properties = self.properties.as_ref().map(|properties| {
            properties
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
                .collect()
        });

        LinkData {
            uri_ref,
            description,
            properties,
        }
    }

    pub fn into_owned(self) -> LinkData<'static> {
        let uri_ref = self.uri_ref.into_owned();
        let description = self.description.map(Description::into_owned);
        let properties = self.properties.map(|properties| {
            properties
                .into_iter()
                .map(|(key, value)| {
                    (Cow::from(key.into_owned()), Cow::from(value.into_owned()))
                })
                .collect()
        });

        LinkData {
            uri_ref,
            description,
            properties,
        }
    }
}

impl<'a> LinkData<'a> {
    /// Returns reference to uri of the link
    pub fn uri_ref(&self) -> &URIReference<'a> {
        &self.uri_ref
    }

    /// Consumes link and returns its uri
    pub fn into_uri_ref(self) -> URIReference<'a> {
        self.uri_ref
    }

    /// Returns reference to the description of the link
    pub fn description(&self) -> Option<&Description<'a>> {
        self.description.as_ref()
    }

    /// Consumes link and returns its description
    pub fn into_description(self) -> Option<Description<'a>> {
        self.description
    }

    /// Returns reference to the properties of the link
    pub fn properties(&self) -> Option<&HashMap<Cow<'a, str>, Cow<'a, str>>> {
        self.properties.as_ref()
    }

    /// Consumes link and returns the properties of the link
    pub fn into_properties(
        self,
    ) -> Option<HashMap<Cow<'a, str>, Cow<'a, str>>> {
        self.properties
    }

    /// Whether or not the link is representing an anchor to the current page
    pub fn is_local_anchor(&self) -> bool {
        self.uri_ref.path().segments().is_empty() && self.has_anchor()
    }

    /// Checks if the link's path is to a directory without actually evaluating
    /// in the filesystem. Only checks if the path appears as that of a
    /// directory
    pub fn is_path_dir(&self) -> bool {
        self.uri_ref
            .path()
            .segments()
            .last()
            .map_or(false, |s| s.as_str().ends_with('/'))
    }

    /// Whether or not the associated URI is local to the current system
    pub fn is_local(&self) -> bool {
        // If we have no scheme, have a file: scheme, or have our custom
        // local: scheme, then the uri's path is local
        self.uri_ref.scheme().map_or(true, |scheme| match scheme {
            Scheme::File => true,
            Scheme::Unregistered(x) if x == "local" => true,
            _ => false,
        })
    }

    /// Whether or not the associated URI is targeting a remote system
    #[inline]
    pub fn is_remote(&self) -> bool {
        !self.is_local()
    }

    /// Produces a `PathBuf` from the path of the link's uri, using the
    /// system's separator as the start if absolute
    pub fn to_path_buf(&self) -> PathBuf {
        let mut path = PathBuf::new();

        if self.uri_ref.path().is_absolute() {
            path.push(std::path::MAIN_SEPARATOR.to_string());
        }

        for seg in self.uri_ref.path().segments() {
            path.push(seg.as_str());
        }

        path
    }

    /// Returns a reference to the fragment portion of the link's uri (after
    /// the first # sign)
    pub fn fragment_str(&self) -> Option<&str> {
        self.uri_ref.fragment().map(Fragment::as_str)
    }

    /// Returns true if the link's uri contains an anchor (#something)
    pub fn has_anchor(&self) -> bool {
        self.uri_ref.has_fragment()
    }

    /// Produces an `Anchor` referencing the fragment portion of the link
    pub fn to_anchor(&self) -> Option<Anchor<'_>> {
        self.fragment_str()
            .map(|s| s.split('#').collect::<Anchor>())
    }

    /// Returns reference to the scheme of the link's uri if it exists
    pub fn scheme(&self) -> Option<&Scheme<'_>> {
        self.uri_ref.scheme()
    }
}

impl<'a> Hash for LinkData<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uri_ref.hash(state);
        self.description.hash(state);

        // Grab all property keys and sort them so we get a reproducible
        // iteration over the keys
        if let Some(properties) = self.properties.as_ref() {
            let mut keys = properties.keys().collect::<Vec<&Cow<'_, str>>>();
            keys.sort_unstable();

            // Use property keys in hash
            for k in keys {
                k.hash(state);
            }
        }
    }
}

impl<'a> fmt::Display for LinkData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = self.description.as_ref() {
            write!(f, "{}", desc)
        } else {
            write!(f, "{}", self.uri_ref)?;
            Ok(())
        }
    }
}

impl TryFrom<String> for LinkData<'static> {
    type Error = URIReferenceError;

    /// Converts String into a link by parsing the String as a `uriparse::URIReference`
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let uri_ref = URIReference::try_from(s.as_str())?.into_owned();
        Ok(Self::new(uri_ref, None, None))
    }
}

impl<'a> TryFrom<&'a str> for LinkData<'a> {
    type Error = URIReferenceError;

    /// Converts str into a link by parsing the str as a `uriparse::URIReference`
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let uri_ref = URIReference::try_from(s)?;
        Ok(Self::new(uri_ref, None, None))
    }
}

impl<'a> From<URIReference<'a>> for LinkData<'a> {
    fn from(uri_ref: URIReference<'a>) -> Self {
        Self::new(uri_ref, None, None)
    }
}

impl<'a> From<LinkData<'a>> for PathBuf {
    fn from(link: LinkData<'a>) -> Self {
        link.to_path_buf()
    }
}

impl<'a> StrictEq for LinkData<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Helper function to borrow a `URIReference` similar to our other approaches as the
/// functionality is not available directly in the `uriparse` crate
fn uri_ref_to_borrowed<'a>(uri_ref: &'a URIReference<'a>) -> URIReference<'a> {
    let scheme = uri_ref.scheme().map(|x| x.as_borrowed());
    let authority = uri_ref.authority().map(|x| x.as_borrowed());
    let query = uri_ref.query().map(|x| x.as_borrowed());
    let fragment = uri_ref.fragment().map(|x| x.as_borrowed());

    // NOTE: Requires an allocation of a new Vec of borrowed elements
    let path = uri_ref.path().to_borrowed();

    URIReference::from_parts(scheme, authority, path, query, fragment)
        .expect("URI failed to borrow itself")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_str_should_succeed_for_empty_str() {
        todo!();
    }

    #[test]
    fn try_from_str_should_succeed_for_anchor_only() {
        todo!();
    }

    #[test]
    fn try_from_str_should_succeed_for_relative_path() {
        todo!();
    }

    #[test]
    fn try_from_str_should_succeed_for_absolute_path() {
        todo!();
    }

    #[test]
    fn try_from_str_should_succeed_for_network_path() {
        todo!();
    }

    #[test]
    fn anchor_should_return_anchor_that_wraps_fragment_pieces() {
        todo!();
    }

    #[test]
    fn to_path_buf_should_return_a_new_path_buf_based_on_uri_path() {
        todo!();
    }

    #[test]
    fn is_local_anchor_should_return_true_if_link_only_has_anchor() {
        todo!();
    }

    #[test]
    fn is_local_anchor_should_return_false_if_has_non_anchor_path() {
        todo!();
    }

    #[test]
    fn is_path_dir_should_return_true_if_link_is_to_directory() {
        todo!();
    }

    #[test]
    fn is_path_idr_should_return_false_if_link_is_not_to_directory() {
        todo!();
    }
}
