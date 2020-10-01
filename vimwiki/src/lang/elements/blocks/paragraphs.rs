use super::{InlineElement, InlineElementContainer, LC};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub content: InlineElementContainer,
}

impl From<Vec<LC<InlineElement>>> for Paragraph {
    fn from(elements: Vec<LC<InlineElement>>) -> Self {
        Self::new(elements.into())
    }
}
