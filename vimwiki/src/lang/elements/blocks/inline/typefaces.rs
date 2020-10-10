use super::{InlineElement, Link, TypedInlineElement, LE};
use derive_more::{AsMut, AsRef, Constructor, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents plain text with no decorations or inline elements
#[derive(
    AsMut,
    AsRef,
    Constructor,
    Clone,
    Debug,
    Display,
    Into,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Text(String);

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Represents content that can be contained within a decoration
#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum DecoratedTextContent {
    Text(TypedInlineElement<Text>),
    Keyword(TypedInlineElement<Keyword>),
    Link(TypedInlineElement<Link>),
}

impl DecoratedTextContent {
    pub fn as_inline_element(&self) -> &InlineElement {
        match self {
            Self::Text(ref x) => x.as_inner(),
            Self::Keyword(ref x) => x.as_inner(),
            Self::Link(ref x) => x.as_inner(),
        }
    }

    pub fn as_mut_inline_element(&mut self) -> &mut InlineElement {
        match self {
            Self::Text(ref mut x) => x.as_mut_inner(),
            Self::Keyword(ref mut x) => x.as_mut_inner(),
            Self::Link(ref mut x) => x.as_mut_inner(),
        }
    }

    pub fn into_inline_element(self) -> InlineElement {
        match self {
            Self::Text(x) => x.into_inner(),
            Self::Keyword(x) => x.into_inner(),
            Self::Link(x) => x.into_inner(),
        }
    }
}

impl From<Text> for DecoratedTextContent {
    fn from(text: Text) -> Self {
        Self::from(TypedInlineElement::from_text(text))
    }
}

impl From<Keyword> for DecoratedTextContent {
    fn from(keyword: Keyword) -> Self {
        Self::from(TypedInlineElement::from_keyword(keyword))
    }
}

impl From<Link> for DecoratedTextContent {
    fn from(link: Link) -> Self {
        Self::from(TypedInlineElement::from_link(link))
    }
}

/// Represents text (series of content) with a typeface decoration
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DecoratedText {
    Bold(Vec<LE<DecoratedTextContent>>),
    Italic(Vec<LE<DecoratedTextContent>>),
    BoldItalic(Vec<LE<DecoratedTextContent>>),
    Strikeout(Vec<LE<DecoratedTextContent>>),
    Superscript(Vec<LE<DecoratedTextContent>>),
    Subscript(Vec<LE<DecoratedTextContent>>),
}

impl DecoratedText {
    /// Converts to the underlying decorated text contents
    pub fn as_contents(&self) -> &[LE<DecoratedTextContent>] {
        match self {
            Self::Bold(ref x) => x.as_slice(),
            Self::Italic(ref x) => x.as_slice(),
            Self::BoldItalic(ref x) => x.as_slice(),
            Self::Strikeout(ref x) => x.as_slice(),
            Self::Superscript(ref x) => x.as_slice(),
            Self::Subscript(ref x) => x.as_slice(),
        }
    }
}

impl fmt::Display for DecoratedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for content in self.as_contents().iter() {
            write!(f, "{}", content.to_string())?;
        }
        Ok(())
    }
}

/// Represents special keywords that have unique syntax highlighting
#[derive(
    Copy, Clone, Debug, Display, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum Keyword {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}
