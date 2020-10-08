use super::Region;
use vimwiki::{elements, LE};

mod code;
pub use code::*;
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
    #[graphql(flatten)]
    Link(Link),
    Tags(Tags),
    Code(CodeInline),
    Math(MathInline),
}

impl From<LE<elements::InlineElement>> for InlineElement {
    fn from(le: LE<elements::InlineElement>) -> Self {
        match le.element {
            elements::InlineElement::Text(x) => {
                Self::from(Text::from(LE::new(x, le.region)))
            }
            elements::InlineElement::DecoratedText(x) => {
                Self::from(DecoratedText::from(LE::new(x, le.region)))
            }
            elements::InlineElement::Keyword(x) => {
                Self::from(Keyword::from(LE::new(x, le.region)))
            }
            elements::InlineElement::Link(x) => {
                Self::from(Link::from(LE::new(x, le.region)))
            }
            elements::InlineElement::Tags(x) => {
                Self::from(Tags::from(LE::new(x, le.region)))
            }
            elements::InlineElement::Code(x) => {
                Self::from(CodeInline::from(LE::new(x, le.region)))
            }
            elements::InlineElement::Math(x) => {
                Self::from(MathInline::from(LE::new(x, le.region)))
            }
        }
    }
}
