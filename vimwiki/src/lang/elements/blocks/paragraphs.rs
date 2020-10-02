use super::{InlineElement, InlineElementContainer, LE};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub content: InlineElementContainer,
}

impl From<Vec<LE<InlineElement>>> for Paragraph {
    fn from(elements: Vec<LE<InlineElement>>) -> Self {
        Self::new(elements.into())
    }
}
