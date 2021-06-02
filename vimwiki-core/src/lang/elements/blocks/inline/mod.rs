use crate::{
    lang::elements::{IntoChildren, Located},
    StrictEq,
};
use derive_more::{
    Constructor, Display, From, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{fmt, iter::FromIterator};

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

impl<'a> IntoChildren for InlineElement<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        match self {
            Self::DecoratedText(x) => x.into_children(),
            _ => vec![],
        }
    }
}

impl<'a> StrictEq for InlineElement<'a> {
    /// Performs strict_eq check on matching inner variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(x), Self::Text(y)) => x.strict_eq(y),
            (Self::DecoratedText(x), Self::DecoratedText(y)) => x.strict_eq(y),
            (Self::Keyword(x), Self::Keyword(y)) => x.strict_eq(y),
            (Self::Link(x), Self::Link(y)) => x.strict_eq(y),
            (Self::Tags(x), Self::Tags(y)) => x.strict_eq(y),
            (Self::Code(x), Self::Code(y)) => x.strict_eq(y),
            (Self::Math(x), Self::Math(y)) => x.strict_eq(y),
            (Self::Comment(x), Self::Comment(y)) => x.strict_eq(y),
            _ => false,
        }
    }
}

/// Represents a convenience wrapper around a series of inline elements
#[derive(
    Constructor,
    Clone,
    Debug,
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
#[into_iterator(owned, ref, ref_mut)]
pub struct InlineElementContainer<'a>(Vec<Located<InlineElement<'a>>>);

impl<'a> InlineElementContainer<'a> {
    /// Returns iterator over references to elements
    pub fn iter(&self) -> impl Iterator<Item = &Located<InlineElement<'a>>> {
        self.into_iter()
    }

    /// Returns iterator over mutable references to elements
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Located<InlineElement<'a>>> {
        self.into_iter()
    }
}

impl InlineElementContainer<'_> {
    pub fn to_borrowed(&self) -> InlineElementContainer {
        let elements = self
            .iter()
            .map(|x| x.as_ref().map(InlineElement::to_borrowed))
            .collect();

        InlineElementContainer::new(elements)
    }

    pub fn into_owned(self) -> InlineElementContainer<'static> {
        let elements = self
            .into_iter()
            .map(|x| x.map(InlineElement::into_owned))
            .collect();

        InlineElementContainer::new(elements)
    }
}

impl<'a> IntoChildren for InlineElementContainer<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.0
    }
}

impl<'a> fmt::Display for InlineElementContainer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for le in self.iter() {
            write!(f, "{}", le.as_inner().to_string())?;
        }
        Ok(())
    }
}

impl<'a> FromIterator<Located<InlineElement<'a>>>
    for InlineElementContainer<'a>
{
    fn from_iter<I: IntoIterator<Item = Located<InlineElement<'a>>>>(
        iter: I,
    ) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<InlineElementContainer<'a>>
    for InlineElementContainer<'a>
{
    fn from_iter<I: IntoIterator<Item = InlineElementContainer<'a>>>(
        iter: I,
    ) -> Self {
        Self::new(iter.into_iter().flat_map(|c| c.0).collect())
    }
}

impl<'a> StrictEq for InlineElementContainer<'a> {
    /// Performs strict_eq check on inner elements
    fn strict_eq(&self, other: &Self) -> bool {
        self.0.strict_eq(&other.0)
    }
}
