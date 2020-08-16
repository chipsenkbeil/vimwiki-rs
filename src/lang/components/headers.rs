use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Header {
    Header1(Header1),
    Header2(Header2),
    Header3(Header3),
    Header4(Header4),
    Header5(Header5),
    Header6(Header6),
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header1 {
    text: String,
}

impl From<&str> for Header1 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header2 {
    text: String,
}

impl From<&str> for Header2 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header3 {
    text: String,
}

impl From<&str> for Header3 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header4 {
    text: String,
}

impl From<&str> for Header4 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header5 {
    text: String,
}

impl From<&str> for Header5 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Header6 {
    text: String,
}

impl From<&str> for Header6 {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}
