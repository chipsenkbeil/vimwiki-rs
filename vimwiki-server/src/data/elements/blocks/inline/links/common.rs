use derive_more::{Constructor, From};
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use vimwiki::{
    self as v,
    vendor::{
        chrono::{self, NaiveDate},
        uriparse::{self, URI},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Description {
    Text(String),
    Uri(Uri),
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(ref x) => write!(f, "{}", x),
            Self::Uri(ref x) => write!(f, "{}", x.to_string()),
        }
    }
}

impl<'a> From<v::Description<'a>> for Description {
    fn from(d: v::Description<'a>) -> Self {
        match d {
            v::Description::Text(x) => Self::Text(x.to_string()),
            v::Description::TransclusionLink(x) => Self::Uri(Uri::from(x.uri)),
        }
    }
}

impl ValueLike for Description {
    fn into_value(self) -> Value {
        match self {
            Self::Text(x) => Value::Text(x),
            Self::Uri(x) => x.into_value(),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        Uri::try_from_value(value)
            .map(Description::Uri)
            .or_else(|value| match value {
                Value::Text(x) => Ok(Self::Text(x)),
                x => Err(x),
            })
    }
}

/// Represents the description of a link
#[async_graphql::Object]
impl Description {
    /// Represents the content of the description if it is text
    async fn text(&self) -> Option<&String> {
        match self {
            Self::Text(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description if it is a URI
    async fn uri(&self) -> Option<&Uri> {
        match self {
            Self::Uri(ref x) => Some(x),
            _ => None,
        }
    }

    /// Represents the content of the description
    async fn content(&self) -> String {
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
            elements: a.elements.iter().map(ToString::to_string).collect(),
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
pub struct Uri(URI<'static>);

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl FromStr for Uri {
    type Err = uriparse::URIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        URI::try_from(s).map(|x| Uri(x.into_owned()))
    }
}

impl ValueLike for Uri {
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

impl<'a> From<URI<'a>> for Uri {
    fn from(uri: URI<'a>) -> Self {
        Self(uri.into_owned())
    }
}

/// Represents a traditional URI
#[async_graphql::Object]
impl Uri {
    /// The authority portion of the URI, if it exists
    async fn authority(&self) -> Option<String> {
        self.0.authority().map(|x| x.to_string())
    }

    /// The fragment portion of the URI, if it exists
    async fn fragment(&self) -> Option<String> {
        self.0.fragment().map(|x| x.to_string())
    }

    /// The host portion of the URI, if it exists
    async fn host(&self) -> Option<String> {
        self.0.host().map(|x| x.to_string())
    }

    /// The password portion of the URI, if it exists
    async fn password(&self) -> Option<String> {
        self.0.password().map(|x| x.to_string())
    }

    /// The path of the URI
    async fn path(&self) -> String {
        self.0.path().to_string()
    }

    /// The port portion of the URI, if it exists
    async fn port(&self) -> Option<i32> {
        self.0.port().map(|x| x as i32)
    }

    /// The query portion of the URI, if it exists
    async fn query(&self) -> Option<String> {
        self.0.query().map(|x| x.to_string())
    }

    /// The scheme of the URI
    async fn scheme(&self) -> String {
        self.0.scheme().to_string()
    }

    /// The username portion of the URI, if it exists
    async fn username(&self) -> Option<String> {
        self.0.username().map(|x| x.to_string())
    }

    /// The entire URI
    async fn text(&self) -> String {
        self.0.to_string()
    }
}
