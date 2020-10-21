use super::Located;
use derive_more::{
    Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::fmt;

mod code;
pub use code::*;
mod comments;
pub use comments::*;
mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

/// Represents elements that can be dropped into other elements
#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum InlineElement<'a> {
    Text(Text<'a>),
    DecoratedText(DecoratedText<'a>),
    Keyword(Keyword),
    Link(Link<'a>),
    Tags(Tags<'a>),
    Code(CodeInline<'a>),
    Math(MathInline<'a>),

    /// Comments exist as inline elements, but do not show up when displaying
    /// an inline element enum
    #[display(fmt = "")]
    Comment(Comment<'a>),
}

impl InlineElement<'_> {
    pub fn to_borrowed(&self) -> InlineElement {
        match self {
            Self::Text(x) => InlineElement::from(x.as_borrowed()),
            Self::DecoratedText(x) => InlineElement::from(x.to_borrowed()),
            Self::Keyword(x) => InlineElement::from(*x),
            Self::Link(x) => InlineElement::from(x.to_borrowed()),
            Self::Tags(x) => InlineElement::from(x.to_borrowed()),
            Self::Code(x) => InlineElement::from(x.as_borrowed()),
            Self::Math(x) => InlineElement::from(x.as_borrowed()),
            Self::Comment(x) => InlineElement::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> InlineElement<'static> {
        match self {
            Self::Text(x) => InlineElement::from(x.into_owned()),
            Self::DecoratedText(x) => InlineElement::from(x.into_owned()),
            Self::Keyword(x) => InlineElement::from(x),
            Self::Link(x) => InlineElement::from(x.into_owned()),
            Self::Tags(x) => InlineElement::from(x.into_owned()),
            Self::Code(x) => InlineElement::from(x.into_owned()),
            Self::Math(x) => InlineElement::from(x.into_owned()),
            Self::Comment(x) => InlineElement::from(x.into_owned()),
        }
    }
}

impl<'a> InlineElement<'a> {
    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        match self {
            Self::DecoratedText(x) => x.into_children(),
            _ => vec![],
        }
    }
}

/// Represents a convenience wrapper around a series of inline elements
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct InlineElementContainer<'a> {
    pub elements: Vec<Located<InlineElement<'a>>>,
}

impl InlineElementContainer<'_> {
    pub fn to_borrowed(&self) -> InlineElementContainer {
        let elements = self
            .elements
            .iter()
            .map(|x| x.as_ref().map(InlineElement::to_borrowed))
            .collect();

        InlineElementContainer { elements }
    }

    pub fn into_owned(self) -> InlineElementContainer<'static> {
        let elements = self
            .elements
            .into_iter()
            .map(|x| x.map(InlineElement::into_owned))
            .collect();

        InlineElementContainer { elements }
    }
}

impl<'a> InlineElementContainer<'a> {
    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        self.elements
    }
}

impl<'a> fmt::Display for InlineElementContainer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for le in self.elements.iter() {
            write!(f, "{}", le.as_inner().to_string())?;
        }
        Ok(())
    }
}

impl<'a> From<Vec<InlineElementContainer<'a>>> for InlineElementContainer<'a> {
    fn from(mut containers: Vec<Self>) -> Self {
        Self::new(containers.drain(..).flat_map(|c| c.elements).collect())
    }
}

impl<'a> From<Located<InlineElement<'a>>> for InlineElementContainer<'a> {
    fn from(element: Located<InlineElement<'a>>) -> Self {
        Self::new(vec![element])
    }
}

impl<'a> From<Located<&'a str>> for InlineElementContainer<'a> {
    fn from(element: Located<&'a str>) -> Self {
        Self::from(element.map(Text::from))
    }
}

macro_rules! container_mapping {
    ($type:ty) => {
        impl<'a> From<$type> for InlineElementContainer<'a> {
            fn from(element: $type) -> Self {
                Self::from(element.map(InlineElement::from))
            }
        }
    };
}

container_mapping!(Located<CodeInline<'a>>);
container_mapping!(Located<MathInline<'a>>);
container_mapping!(Located<Text<'a>>);
container_mapping!(Located<DecoratedText<'a>>);
container_mapping!(Located<Keyword>);
container_mapping!(Located<Link<'a>>);
container_mapping!(Located<Tags<'a>>);
