use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Header {
    Header1(Header1),
    Header2(Header2),
    Header3(Header3),
    Header4(Header4),
    Header5(Header5),
    Header6(Header6),
}

impl Header {
    pub fn level(&self) -> usize {
        match self {
            Self::Header1(_) => 1,
            Self::Header2(_) => 2,
            Self::Header3(_) => 3,
            Self::Header4(_) => 4,
            Self::Header5(_) => 5,
            Self::Header6(_) => 6,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Header1(h) => &h.text,
            Self::Header2(h) => &h.text,
            Self::Header3(h) => &h.text,
            Self::Header4(h) => &h.text,
            Self::Header5(h) => &h.text,
            Self::Header6(h) => &h.text,
        }
    }

    pub fn is_centered(&self) -> bool {
        match self {
            Self::Header1(h) => h.centered,
            Self::Header2(h) => h.centered,
            Self::Header3(h) => h.centered,
            Self::Header4(h) => h.centered,
            Self::Header5(h) => h.centered,
            Self::Header6(h) => h.centered,
        }
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header1 {
    text: String,
    centered: bool,
}

impl From<&str> for Header1 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header1 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header2 {
    text: String,
    centered: bool,
}

impl From<&str> for Header2 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header2 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header3 {
    text: String,
    centered: bool,
}

impl From<&str> for Header3 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header3 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header4 {
    text: String,
    centered: bool,
}

impl From<&str> for Header4 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header4 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header5 {
    text: String,
    centered: bool,
}

impl From<&str> for Header5 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header5 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header6 {
    text: String,
    centered: bool,
}

impl From<&str> for Header6 {
    fn from(s: &str) -> Self {
        Self::from((s, false))
    }
}

impl From<(&str, bool)> for Header6 {
    fn from(input: (&str, bool)) -> Self {
        Self::new(input.0.to_string(), input.1)
    }
}
