use super::{InlineComponent, InlineComponentContainer, LC};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub content: InlineComponentContainer,
}

impl From<Vec<LC<InlineComponent>>> for Paragraph {
    fn from(components: Vec<LC<InlineComponent>>) -> Self {
        Self::new(components.into())
    }
}
