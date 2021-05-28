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
    pub lines: Vec<InlineElementContainer<'a>>,
}

impl Paragraph<'_> {
    pub fn to_borrowed(&self) -> Paragraph {
        Paragraph {
            lines: self.lines.iter().map(|x| x.to_borrowed()).collect(),
        }
    }

    pub fn into_owned(self) -> Paragraph<'static> {
        Paragraph {
            lines: self.lines.into_iter().map(|x| x.into_owned()).collect(),
        }
    }
}

impl<'a> IntoChildren for Paragraph<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.lines
            .into_iter()
            .flat_map(|x| x.into_children())
            .collect()
    }
}

impl<'a> From<Vec<Located<InlineElement<'a>>>> for Paragraph<'a> {
    /// Wraps multiple located inline elements in a container that is then
    /// placed inside a paragraph
    fn from(elements: Vec<Located<InlineElement<'a>>>) -> Self {
        Self::new(vec![elements.into()])
    }
}

impl<'a> From<Located<InlineElement<'a>>> for Paragraph<'a> {
    /// Wraps single, located inline element in a container that is then
    /// placed inside a paragraph
    fn from(element: Located<InlineElement<'a>>) -> Self {
        Self::new(vec![element.into()])
    }
}

impl<'a> StrictEq for Paragraph<'a> {
    /// Performs strict_eq on content
    fn strict_eq(&self, other: &Self) -> bool {
        self.lines.strict_eq(&other.lines)
    }
}
