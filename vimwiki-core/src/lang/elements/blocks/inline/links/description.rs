use super::LinkData;
use crate::StrictEq;
use derive_more::{Display, From, IsVariant, TryInto};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, convert::TryFrom};
use uriparse::{URIReference, URIReferenceError};

/// Represents a description for a link
#[derive(
    Clone,
    Debug,
    Display,
    From,
    TryInto,
    Eq,
    PartialEq,
    Hash,
    IsVariant,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Description<'a> {
    Text(Cow<'a, str>),
    TransclusionLink(Box<LinkData<'a>>),
}

impl<'a> Description<'a> {
    pub fn into_uri_ref(self) -> Option<URIReference<'a>> {
        match self {
            Self::TransclusionLink(x) => Some(x.uri_ref),
            _ => None,
        }
    }

    pub fn try_from_uri_ref_str(
        s: &'a str,
    ) -> Result<Description<'a>, URIReferenceError> {
        Ok(Description::TransclusionLink(Box::new(LinkData::try_from(
            s,
        )?)))
    }
}

impl Description<'_> {
    pub fn to_borrowed(&self) -> Description {
        use self::Cow::*;

        match self {
            Self::Text(ref x) => Description::from(Cow::Borrowed(match x {
                Borrowed(x) => *x,
                Owned(x) => x.as_str(),
            })),
            Self::TransclusionLink(ref x) => Description::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Description<'static> {
        match self {
            Self::Text(x) => Description::from(Cow::from(x.into_owned())),
            Self::TransclusionLink(x) => Description::from(x.into_owned()),
        }
    }
}

impl<'a> From<&'a str> for Description<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Cow::Borrowed(s))
    }
}

impl From<String> for Description<'static> {
    fn from(s: String) -> Self {
        Self::from(Cow::Owned(s))
    }
}

impl<'a> From<URIReference<'a>> for Description<'a> {
    fn from(uri_ref: URIReference<'a>) -> Self {
        Self::from(LinkData::from(uri_ref))
    }
}

impl<'a> From<LinkData<'a>> for Description<'a> {
    fn from(link_data: LinkData<'a>) -> Self {
        Self::TransclusionLink(Box::new(link_data))
    }
}

impl<'a> StrictEq for Description<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
