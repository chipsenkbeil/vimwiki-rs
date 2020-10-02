use super::{Link, LE};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

/// Represents a typeface decoration that can be applied to text
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    Bold,
    Italic,
    BoldItalic,
    Strikeout,
    Code,
    Superscript,
    Subscript,
}

/// Represents content that can be contained within a decoration
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DecoratedTextContent {
    Text(String),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
}

/// Represents text (series of content) with a typeface decoration
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct DecoratedText {
    pub contents: Vec<LE<DecoratedTextContent>>,
    pub decoration: Decoration,
}

/// Represents special keywords that have unique syntax highlighting
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}
