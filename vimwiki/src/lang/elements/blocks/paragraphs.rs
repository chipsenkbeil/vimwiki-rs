use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph<'a> {
    pub content: InlineElementContainer<'a>,
}

impl Paragraph<'_> {
    pub fn to_borrowed(&self) -> Paragraph {
        Paragraph {
            content: self.content.to_borrowed(),
        }
    }

    pub fn into_owned(self) -> Paragraph<'static> {
        Paragraph {
            content: self.content.into_owned(),
        }
    }
}

impl<'a> IntoChildren for Paragraph<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.content.into_children()
    }
}

impl<'a> From<Vec<Located<InlineElement<'a>>>> for Paragraph<'a> {
    /// Wraps multiple located inline elements in a container that is then
    /// placed inside a paragraph
    fn from(elements: Vec<Located<InlineElement<'a>>>) -> Self {
        Self::new(elements.into())
    }
}

impl<'a> From<Located<InlineElement<'a>>> for Paragraph<'a> {
    /// Wraps single, located inline element in a container that is then
    /// placed inside a paragraph
    fn from(element: Located<InlineElement<'a>>) -> Self {
        Self::new(element.into())
    }
}

impl<'a> StrictEq for Paragraph<'a> {
    /// Performs strict_eq on content
    fn strict_eq(&self, other: &Self) -> bool {
        self.content.strict_eq(&other.content)
    }
}
