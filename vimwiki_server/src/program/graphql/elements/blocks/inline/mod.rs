use super::Region;
use vimwiki::{elements, LE};

mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

#[derive(async_graphql::Union, Debug)]
pub enum InlineElement {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    #[item(flatten)]
    Link(Link),
    Tags(Tags),
    Math(MathInline),
}

impl From<LE<elements::InlineElement>> for InlineElement {
    fn from(lc: LE<elements::InlineElement>) -> Self {
        match lc.element {
            elements::InlineElement::Text(x) => {
                Self::from(Text::from(LE::new(x, lc.region)))
            }
            elements::InlineElement::DecoratedText(x) => {
                Self::from(DecoratedText::from(LE::new(x, lc.region)))
            }
            elements::InlineElement::Keyword(x) => {
                Self::from(Keyword::from(LE::new(x, lc.region)))
            }
            elements::InlineElement::Link(x) => {
                Self::from(Link::from(LE::new(x, lc.region)))
            }
            elements::InlineElement::Tags(x) => {
                Self::from(Tags::from(LE::new(x, lc.region)))
            }
            elements::InlineElement::Math(x) => {
                Self::from(MathInline::from(LE::new(x, lc.region)))
            }
        }
    }
}
