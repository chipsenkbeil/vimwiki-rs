use crate::lang::elements::{InlineElement, Link, Located, TypedInlineElement};
use derive_more::{AsMut, AsRef, Constructor, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

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
pub struct Text<'a>(pub Cow<'a, str>);

impl From<String> for Text<'static> {
    fn from(s: String) -> Self {
        Self::new(Cow::from(s))
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}

/// Represents content that can be contained within a decoration
#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum DecoratedTextContent<'a> {
    Text(TypedInlineElement<'a, Text<'a>>),
    Keyword(TypedInlineElement<'a, Keyword>),
    Link(TypedInlineElement<'a, Link<'a>>),
}

impl<'a> DecoratedTextContent<'a> {
    pub fn as_inline_element(&'a self) -> &InlineElement<'a> {
        match self {
            Self::Text(ref x) => x.as_inner(),
            Self::Keyword(ref x) => x.as_inner(),
            Self::Link(ref x) => x.as_inner(),
        }
    }

    pub fn as_mut_inline_element(&mut self) -> &mut InlineElement<'a> {
        match self {
            Self::Text(ref mut x) => x.as_mut_inner(),
            Self::Keyword(ref mut x) => x.as_mut_inner(),
            Self::Link(ref mut x) => x.as_mut_inner(),
        }
    }

    pub fn into_inline_element(self) -> InlineElement<'a> {
        match self {
            Self::Text(x) => x.into_inner(),
            Self::Keyword(x) => x.into_inner(),
            Self::Link(x) => x.into_inner(),
        }
    }
}

impl<'a> From<Text<'a>> for DecoratedTextContent<'a> {
    fn from(text: Text<'a>) -> Self {
        Self::from(TypedInlineElement::from_text(text))
    }
}

impl From<Keyword> for DecoratedTextContent<'static> {
    fn from(keyword: Keyword) -> Self {
        Self::from(TypedInlineElement::from_keyword(keyword))
    }
}

impl<'a> From<Link<'a>> for DecoratedTextContent<'a> {
    fn from(link: Link<'a>) -> Self {
        Self::from(TypedInlineElement::from_link(link))
    }
}

/// Represents text (series of content) with a typeface decoration
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DecoratedText<'a> {
    Bold(Vec<Located<DecoratedTextContent<'a>>>),
    Italic(Vec<Located<DecoratedTextContent<'a>>>),
    BoldItalic(Vec<Located<DecoratedTextContent<'a>>>),
    Strikeout(Vec<Located<DecoratedTextContent<'a>>>),
    Superscript(Vec<Located<DecoratedTextContent<'a>>>),
    Subscript(Vec<Located<DecoratedTextContent<'a>>>),
}

impl<'a> DecoratedText<'a> {
    /// Converts to the underlying decorated text contents
    pub fn as_contents(&self) -> &[Located<DecoratedTextContent<'a>>] {
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

impl<'a> fmt::Display for DecoratedText<'a> {
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
