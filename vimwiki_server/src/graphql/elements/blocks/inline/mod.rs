use super::Region;
use vimwiki::{elements, LC};

mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

#[derive(async_graphql::Union)]
pub enum InlineElement {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    #[item(flatten)]
    Link(Link),
    Tags(Tags),
    Math(MathInline),
}

impl From<LC<elements::InlineElement>> for InlineElement {
    fn from(lc: LC<elements::InlineElement>) -> Self {
        match lc.element {
            elements::InlineElement::Text(x) => {
                Self::from(Text::from(LC::new(x, lc.region)))
            }
            elements::InlineElement::DecoratedText(x) => {
                Self::from(DecoratedText::from(LC::new(x, lc.region)))
            }
            elements::InlineElement::Keyword(x) => {
                Self::from(Keyword::from(LC::new(x, lc.region)))
            }
            elements::InlineElement::Link(x) => {
                Self::from(Link::from(LC::new(x, lc.region)))
            }
            elements::InlineElement::Tags(x) => {
                Self::from(Tags::from(LC::new(x, lc.region)))
            }
            elements::InlineElement::Math(x) => {
                Self::from(MathInline::from(LC::new(x, lc.region)))
            }
        }
    }
}
