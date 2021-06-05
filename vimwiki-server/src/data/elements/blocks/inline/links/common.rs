use derive_more::{Constructor, From};
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use vimwiki::{
    self as v,
    vendor::{
        chrono::{self, NaiveDate},
        uriparse::{self, URIReference},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Description {
    Text(String),
    UriRef(UriRef),
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(ref x) => write!(f, "{}", x),
            Self::UriRef(ref x) => write!(f, "{}", x.to_string()),
        }
    }
}

impl<'a> From<v::Description<'a>> for Description {
    fn from(d: v::Description<'a>) -> Self {
        match d {
            v::Description::Text(x) => Self::Text(x.to_string()),
            v::Description::TransclusionLink(x) => {
                Self::UriRef(UriRef::from(x.uri_ref))
            }
        }
    }
}

impl ValueLike for Description {
    fn into_value(self) -> Value {
        match self {
            Self::Text(x) => Value::Text(x),
            Self::UriRef(x) => x.into_value(),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        UriRef::try_from_value(value)
            .map(Description::UriRef)
            .or_else(|value| match value {
                Value::Text(x) => Ok(Self::Text(x)),
                x => Err(x),
            })
    }
}

/// Represents the description of a link
#[async_graphql::Object]
impl Description {
    /// Represents the content of the description if it is a URI
    async fn uri_ref(&self) -> Option<&UriRef> {
        match self {
            Self::UriRef(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description as text
    async fn text(&self) -> String {
        self.to_string()
    }
}

/// Represents anchor for a link
#[derive(
    async_graphql::SimpleObject,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    ValueLike,
)]
pub struct Anchor {
    /// The pieces of an anchor #one#two#three -> ["one", "two", "three"]
    elements: Vec<String>,
}

impl Anchor {
    pub fn new<S: Into<String>, I: IntoIterator<Item = S>>(
        elements: I,
    ) -> Self {
        Self {
            elements: elements.into_iter().map(Into::into).collect(),
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in self.elements.iter() {
            write!(f, "#{}", e.as_str())?;
        }
        Ok(())
    }
}

impl<'a> From<v::Anchor<'a>> for Anchor {
    fn from(a: v::Anchor<'a>) -> Self {
        Self {
            elements: a.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, From, Serialize, Deserialize)]
pub struct Date(NaiveDate);

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d").to_string())
    }
}

impl FromStr for Date {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(&s, "%Y-%m-%d").map(Date)
    }
}

impl ValueLike for Date {
    fn into_value(self) -> Value {
        Value::Text(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}

async_graphql::scalar!(Date);

#[derive(Constructor, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UriRef(URIReference<'static>);

impl fmt::Display for UriRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl FromStr for UriRef {
    type Err = uriparse::URIReferenceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        URIReference::try_from(s).map(|x| UriRef(x.into_owned()))
    }
}

impl ValueLike for UriRef {
    fn into_value(self) -> Value {
        Value::Text(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}

impl<'a> From<URIReference<'a>> for UriRef {
    fn from(uri_ref: URIReference<'a>) -> Self {
        Self(uri_ref.into_owned())
    }
}

/// Represents a traditional URI (or relative reference)
#[async_graphql::Object]
impl UriRef {
    /// The authority portion of the URI, if it exists
    async fn authority(&self) -> Option<String> {
        self.0.authority().map(ToString::to_string)
    }

    /// The fragment portion of the URI, if it exists
    async fn fragment(&self) -> Option<String> {
        self.0.fragment().map(ToString::to_string)
    }

    /// The host portion of the URI, if it exists
    async fn host(&self) -> Option<String> {
        self.0.host().map(ToString::to_string)
    }

    /// The password portion of the URI, if it exists
    async fn password(&self) -> Option<String> {
        self.0.password().map(ToString::to_string)
    }

    /// The path of the URI
    async fn path(&self) -> String {
        self.0.path().to_string()
    }

    /// The port portion of the URI, if it exists
    async fn port(&self) -> Option<u16> {
        self.0.port()
    }

    /// The query portion of the URI, if it exists
    async fn query(&self) -> Option<String> {
        self.0.query().map(ToString::to_string)
    }

    /// The scheme of the URI
    async fn scheme(&self) -> Option<String> {
        self.0.scheme().map(ToString::to_string)
    }

    /// The username portion of the URI, if it exists
    async fn username(&self) -> Option<String> {
        self.0.username().map(ToString::to_string)
    }

    /// The entire URI as a textual representation
    async fn text(&self) -> String {
        self.0.to_string()
    }
}
