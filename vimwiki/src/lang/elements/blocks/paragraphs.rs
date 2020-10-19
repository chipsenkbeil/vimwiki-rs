use super::{InlineElement, InlineElementContainer, Located};
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

impl<'a> Paragraph<'a> {
    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        self.content.into_children()
    }
}

impl<'a> From<Vec<Located<InlineElement<'a>>>> for Paragraph<'a> {
    fn from(elements: Vec<Located<InlineElement<'a>>>) -> Self {
        Self::new(elements.into())
    }
}
