use super::Region;
use vimwiki::{components, LC};

mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

#[derive(async_graphql::Union)]
pub enum InlineComponent {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    Tags(Tags),
    Math(MathInline),
}

impl From<LC<components::InlineComponent>> for InlineComponent {
    fn from(lc: LC<components::InlineComponent>) -> Self {
        match lc.component {
            components::InlineComponent::Text(x) => {
                Self::from(Text::from(LC::new(x, lc.region)))
            }
            components::InlineComponent::DecoratedText(x) => {
                Self::from(DecoratedText::from(LC::new(x, lc.region)))
            }
            components::InlineComponent::Keyword(x) => {
                Self::from(Keyword::from(LC::new(x, lc.region)))
            }
            components::InlineComponent::Link(x) => {
                Self::from(Link::from(LC::new(x, lc.region)))
            }
            components::InlineComponent::Tags(x) => {
                Self::from(Tags::from(LC::new(x, lc.region)))
            }
            components::InlineComponent::Math(x) => {
                Self::from(MathInline::from(LC::new(x, lc.region)))
            }
        }
    }
}
