use super::{InlineComponent, InlineComponentContainer};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub content: InlineComponentContainer,
}

impl From<Vec<InlineComponent>> for Paragraph {
    fn from(components: Vec<InlineComponent>) -> Self {
        Self::new(components.into())
    }
}
