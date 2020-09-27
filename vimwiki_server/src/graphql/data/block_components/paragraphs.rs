use super::{InlineComponent, Region};
use vimwiki::{components, LC};

#[derive(async_graphql::SimpleObject)]
pub struct Paragraph {
    region: Region,
    components: Vec<InlineComponent>,
}

impl From<LC<components::Paragraph>> for Paragraph {
    fn from(mut lc: LC<components::Paragraph>) -> Self {
        Self {
            region: Region::from(lc.region),
            components: lc
                .component
                .content
                .components
                .drain(..)
                .map(InlineComponent::from)
                .collect(),
        }
    }
}
