use crate::StrictEq;
use chrono::NaiveDate;
use derive_more::Display;
use percent_encoding::percent_decode;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, convert::TryFrom};
use uriparse::{Scheme, URIReference};

mod anchor;
pub use anchor::Anchor;

mod description;
pub use description::Description;

mod data;
pub use data::LinkData;

/// Represents some kind of link in a document
#[derive(
    Clone, Debug, Display, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum Link<'a> {
    /// Represents a traditional link within a wiki
    #[display(fmt = "{}", data)]
    Wiki { data: LinkData<'a> },

    /// Represents a link to another wiki referenced by its index
    #[display(fmt = "{}", data)]
    IndexedInterWiki { index: u32, data: LinkData<'a> },

    /// Represents a link to another wiki referenced by its name
    #[display(fmt = "{}", data)]
    NamedInterWiki {
        name: Cow<'a, str>,
        data: LinkData<'a>,
    },

    /// Represents a link to a diary entry within a wiki
    #[display(fmt = "{}", "data.description().map(ToString::to_string).unwrap_or_else(|| date.to_string())")]
    Diary { date: NaiveDate, data: LinkData<'a> },

    /// Represents a raw link without any frills (should only have URI)
    #[display(fmt = "{}", data)]
    Raw { data: LinkData<'a> },

    /// Represents a transclusion link that is used to pull in the content
    /// referenced by the URI
    #[display(fmt = "{}", data)]
    Transclusion { data: LinkData<'a> },
}

impl<'a> Link<'a> {
    /// Creates a new wiki link
    pub fn new_wiki_link<
        U: Into<URIReference<'a>>,
        D: Into<Option<Description<'a>>>,
    >(
        uri_ref: U,
        description: D,
    ) -> Self {
        Self::Wiki {
            data: LinkData::new(uri_ref.into(), description.into(), None),
        }
    }

    /// Creates a new indexed interwiki link
    pub fn new_indexed_interwiki_link<
        U: Into<URIReference<'a>>,
        D: Into<Option<Description<'a>>>,
    >(
        index: u32,
        uri_ref: U,
        description: D,
    ) -> Self {
        Self::IndexedInterWiki {
            index,
            data: LinkData::new(uri_ref.into(), description.into(), None),
        }
    }

    /// Creates a new named interwiki link
    pub fn new_named_interwiki_link<
        S: Into<Cow<'a, str>>,
        U: Into<URIReference<'a>>,
        D: Into<Option<Description<'a>>>,
    >(
        name: S,
        uri_ref: U,
        description: D,
    ) -> Self {
        Self::NamedInterWiki {
            name: name.into(),
            data: LinkData::new(uri_ref.into(), description.into(), None),
        }
    }

    /// Creates a new diary link
    pub fn new_diary_link<
        D: Into<Option<Description<'a>>>,
        A: Into<Option<Anchor<'a>>>,
    >(
        date: NaiveDate,
        description: D,
        anchor: A,
    ) -> Self {
        // NOTE: Based on provided anchor, we produce a URI-compatible anchor
        let empty_uri_ref = URIReference::try_from(
            anchor
                .into()
                .as_ref()
                .map_or_else(String::new, Anchor::to_encoded_uri_fragment)
                .as_str(),
        )
        .unwrap()
        .into_owned();

        Self::Diary {
            date,
            data: LinkData::new(empty_uri_ref, description.into(), None),
        }
    }

    /// Creates a new raw link
    pub fn new_raw_link<U: Into<URIReference<'a>>>(uri_ref: U) -> Self {
        Self::Raw {
            data: LinkData::from(uri_ref.into()),
        }
    }

    /// Creates a new transclusion link
    pub fn new_transclusion_link<
        U: Into<URIReference<'a>>,
        D: Into<Option<Description<'a>>>,
        P: Into<Option<HashMap<Cow<'a, str>, Cow<'a, str>>>>,
    >(
        uri_ref: U,
        description: D,
        properties: P,
    ) -> Self {
        Self::Transclusion {
            data: LinkData::new(
                uri_ref.into(),
                description.into(),
                properties.into(),
            ),
        }
    }

    /// Returns reference to data associated with link
    pub fn data(&self) -> &LinkData<'a> {
        match self {
            Self::Wiki { data } => data,
            Self::IndexedInterWiki { data, .. } => data,
            Self::NamedInterWiki { data, .. } => data,
            Self::Diary { data, .. } => data,
            Self::Raw { data } => data,
            Self::Transclusion { data } => data,
        }
    }

    /// Consumes link and returns data associated with link
    pub fn into_data(self) -> LinkData<'a> {
        match self {
            Self::Wiki { data } => data,
            Self::IndexedInterWiki { data, .. } => data,
            Self::NamedInterWiki { data, .. } => data,
            Self::Diary { data, .. } => data,
            Self::Raw { data } => data,
            Self::Transclusion { data } => data,
        }
    }

    /// Returns reference to description associated with link
    pub fn description(&self) -> Option<&Description<'a>> {
        self.data().description()
    }

    /// Consumes link and returns the description associated with link
    pub fn into_description(self) -> Option<Description<'a>> {
        self.into_data().into_description()
    }

    /// Produces a description based on the link, either using the description
    /// associated with the link or falling back to the link's URI after
    /// applying percent decoding (not for raw or transclusion links)
    pub fn to_description_or_fallback(&self) -> Option<Description<'a>> {
        if let Some(desc) = self.description() {
            // If we have an actual description, just return it
            Some(desc.clone())
        } else if matches!(self, Link::Raw { .. } | Link::Transclusion { .. }) {
            // If a raw link or transclusion, we don't want to infer a
            // description if one is not there
            None
        } else if let Link::Diary { date, .. } = self {
            // Diary links have no uri but instead use a date
            Some(Description::from(format!(
                "diary:{}",
                date.format("%Y-%m-%d")
            )))
        } else {
            // If not a raw link, we want to make sure to clean up %20 and
            // other percent encoded pieces
            Some(Description::from(
                percent_decode(self.data().uri_ref().to_string().as_bytes())
                    .decode_utf8_lossy()
                    .to_string(),
            ))
        }
    }

    /// Returns reference to the properties associated with link
    pub fn properties(&self) -> Option<&HashMap<Cow<'a, str>, Cow<'a, str>>> {
        self.data().properties()
    }

    /// Consumes link and returns the properties associated with link
    pub fn into_properties(
        self,
    ) -> Option<HashMap<Cow<'a, str>, Cow<'a, str>>> {
        self.into_data().into_properties()
    }

    /// Returns true if link contains an anchor
    pub fn has_anchor(&self) -> bool {
        self.data().has_anchor()
    }

    /// Creates anchor based on link's uri fragment if it exists
    pub fn to_anchor(&self) -> Option<Anchor<'_>> {
        self.data().to_anchor()
    }

    /// Returns reference to the scheme of the link's uri if it exists
    pub fn scheme(&self) -> Option<&Scheme<'_>> {
        self.data().scheme()
    }

    /// Returns a copy of the date associated with the link if it exists
    /// (only occurs when is a diary link)
    pub fn date(&self) -> Option<NaiveDate> {
        match self {
            Self::Diary { date, .. } => Some(*date),
            _ => None,
        }
    }

    /// Returns a copy of the index associated with the link if it exist
    /// (only occurs when is an indexed interwiki link)
    pub fn index(&self) -> Option<u32> {
        match self {
            Self::IndexedInterWiki { index, .. } => Some(*index),
            _ => None,
        }
    }

    /// Returns a reference to the name associated with the link if it exists
    /// (only occurs when is a named interwiki link)
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::NamedInterWiki { name, .. } => Some(name.as_ref()),
            _ => None,
        }
    }
}

impl Link<'_> {
    pub fn to_borrowed(&self) -> Link {
        use self::Cow::*;

        match self {
            Self::Wiki { data } => Link::Wiki {
                data: data.to_borrowed(),
            },
            Self::IndexedInterWiki { index, data } => Link::IndexedInterWiki {
                index: *index,
                data: data.to_borrowed(),
            },
            Self::NamedInterWiki { name, data } => Link::NamedInterWiki {
                name: Borrowed(match name {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                }),
                data: data.to_borrowed(),
            },
            Self::Diary { date, data } => Link::Diary {
                date: *date,
                data: data.to_borrowed(),
            },
            Self::Raw { data } => Link::Raw {
                data: data.to_borrowed(),
            },
            Self::Transclusion { data } => Link::Transclusion {
                data: data.to_borrowed(),
            },
        }
    }

    pub fn into_owned(self) -> Link<'static> {
        use self::Cow::*;

        match self {
            Self::Wiki { data } => Link::Wiki {
                data: data.into_owned(),
            },
            Self::IndexedInterWiki { index, data } => Link::IndexedInterWiki {
                index,
                data: data.into_owned(),
            },
            Self::NamedInterWiki { name, data } => Link::NamedInterWiki {
                name: Owned(name.into_owned()),
                data: data.into_owned(),
            },
            Self::Diary { date, data } => Link::Diary {
                date,
                data: data.into_owned(),
            },
            Self::Raw { data } => Link::Raw {
                data: data.into_owned(),
            },
            Self::Transclusion { data } => Link::Transclusion {
                data: data.into_owned(),
            },
        }
    }
}

impl<'a> StrictEq for Link<'a> {
    /// Performs strict_eq check on matching inner variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Wiki { data: a }, Self::Wiki { data: b }) => a.strict_eq(b),
            (
                Self::IndexedInterWiki {
                    index: a1,
                    data: a2,
                },
                Self::IndexedInterWiki {
                    index: b1,
                    data: b2,
                },
            ) => a1 == b1 && a2.strict_eq(b2),
            (
                Self::NamedInterWiki { name: a1, data: a2 },
                Self::NamedInterWiki { name: b1, data: b2 },
            ) => a1 == b1 && a2.strict_eq(b2),
            (
                Self::Diary { date: a1, data: a2 },
                Self::Diary { date: b1, data: b2 },
            ) => a1 == b1 && a2.strict_eq(b2),
            (Self::Raw { data: a }, Self::Raw { data: b }) => a.strict_eq(b),
            (
                Self::Transclusion { data: a },
                Self::Transclusion { data: b },
            ) => a.strict_eq(b),
            _ => false,
        }
    }
}
