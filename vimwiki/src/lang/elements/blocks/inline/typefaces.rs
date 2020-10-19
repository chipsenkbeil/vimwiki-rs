use crate::lang::elements::{InlineElement, Link, Located};
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

impl Text<'_> {
    pub fn as_borrowed(&self) -> Text {
        use self::Cow::*;

        let inner = Cow::Borrowed(match &self.0 {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        Text(inner)
    }

    pub fn into_owned(self) -> Text<'static> {
        let inner = Cow::from(self.0.into_owned());

        Text(inner)
    }
}

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
    Text(Text<'a>),
    DecoratedText(DecoratedText<'a>),
    Keyword(Keyword),
    Link(Link<'a>),
}

impl DecoratedTextContent<'_> {
    pub fn to_borrowed(&self) -> DecoratedTextContent {
        match self {
            Self::Text(x) => DecoratedTextContent::from(x.as_borrowed()),
            Self::DecoratedText(x) => {
                DecoratedTextContent::from(x.to_borrowed())
            }
            Self::Keyword(x) => DecoratedTextContent::from(*x),
            Self::Link(x) => DecoratedTextContent::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> DecoratedTextContent<'static> {
        match self {
            Self::Text(x) => DecoratedTextContent::from(x.into_owned()),
            Self::DecoratedText(x) => {
                DecoratedTextContent::from(x.into_owned())
            }
            Self::Keyword(x) => DecoratedTextContent::from(x),
            Self::Link(x) => DecoratedTextContent::from(x.into_owned()),
        }
    }
}

impl<'a> DecoratedTextContent<'a> {
    /// Borrows the content and wraps it in an `InlineElement`
    pub fn to_inline_element(&'a self) -> InlineElement<'a> {
        match self {
            Self::Text(ref x) => x.as_borrowed().into(),
            Self::DecoratedText(ref x) => x.to_borrowed().into(),
            Self::Keyword(x) => (*x).into(),
            Self::Link(ref x) => x.to_borrowed().into(),
        }
    }

    /// Consumes content, producing the result wrapped in an `InlineElement`
    pub fn into_inline_element(self) -> InlineElement<'a> {
        match self {
            Self::Text(x) => x.into(),
            Self::DecoratedText(x) => x.into(),
            Self::Keyword(x) => x.into(),
            Self::Link(x) => x.into(),
        }
    }

    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        match self {
            Self::DecoratedText(x) => x.into_children(),
            _ => vec![],
        }
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

impl DecoratedText<'_> {
    pub fn to_borrowed(&self) -> DecoratedText {
        macro_rules! vec_to_borrowed {
            ($vec:expr) => {
                $vec.iter()
                    .map(|x| x.as_ref().map(DecoratedTextContent::to_borrowed))
                    .collect()
            };
        }

        match self {
            Self::Bold(x) => DecoratedText::Bold(vec_to_borrowed!(x)),
            Self::Italic(x) => DecoratedText::Italic(vec_to_borrowed!(x)),
            Self::BoldItalic(x) => {
                DecoratedText::BoldItalic(vec_to_borrowed!(x))
            }
            Self::Strikeout(x) => DecoratedText::Strikeout(vec_to_borrowed!(x)),
            Self::Superscript(x) => {
                DecoratedText::Superscript(vec_to_borrowed!(x))
            }
            Self::Subscript(x) => DecoratedText::Subscript(vec_to_borrowed!(x)),
        }
    }

    pub fn into_owned(self) -> DecoratedText<'static> {
        macro_rules! vec_into_owned {
            ($vec:expr) => {
                $vec.into_iter()
                    .map(|x| x.map(DecoratedTextContent::into_owned))
                    .collect()
            };
        }

        match self {
            Self::Bold(x) => DecoratedText::Bold(vec_into_owned!(x)),
            Self::Italic(x) => DecoratedText::Italic(vec_into_owned!(x)),
            Self::BoldItalic(x) => {
                DecoratedText::BoldItalic(vec_into_owned!(x))
            }
            Self::Strikeout(x) => DecoratedText::Strikeout(vec_into_owned!(x)),
            Self::Superscript(x) => {
                DecoratedText::Superscript(vec_into_owned!(x))
            }
            Self::Subscript(x) => DecoratedText::Subscript(vec_into_owned!(x)),
        }
    }
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

    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        macro_rules! vec_to_owned {
            ($vec:expr) => {
                $vec.into_iter()
                    .map(|x| x.map(DecoratedTextContent::into_inline_element))
                    .collect()
            };
        }
        match self {
            Self::Bold(x) => vec_to_owned!(x),
            Self::Italic(x) => vec_to_owned!(x),
            Self::BoldItalic(x) => vec_to_owned!(x),
            Self::Strikeout(x) => vec_to_owned!(x),
            Self::Superscript(x) => vec_to_owned!(x),
            Self::Subscript(x) => vec_to_owned!(x),
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
