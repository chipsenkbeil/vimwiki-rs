use super::Region;
use vimwiki::{elements, Located};

mod code;
pub use code::*;
mod comments;
pub use comments::*;
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
    #[graphql(flatten)]
    Comment(Comment),
}

impl<'a> From<Located<elements::InlineElement<'a>>> for InlineElement {
    fn from(le: Located<elements::InlineElement<'a>>) -> Self {
        let region = le.region();
        match le.into_inner() {
            elements::InlineElement::Text(x) => {
                Self::from(Text::from(Located::new(x, region)))
            }
            elements::InlineElement::DecoratedText(x) => {
                Self::from(DecoratedText::from(Located::new(x, region)))
            }
            elements::InlineElement::Keyword(x) => {
                Self::from(Keyword::from(Located::new(x, region)))
            }
            elements::InlineElement::Link(x) => {
                Self::from(Link::from(Located::new(x, region)))
            }
            elements::InlineElement::Tags(x) => {
                Self::from(Tags::from(Located::new(x, region)))
            }
            elements::InlineElement::Code(x) => {
                Self::from(CodeInline::from(Located::new(x, region)))
            }
            elements::InlineElement::Math(x) => {
                Self::from(MathInline::from(Located::new(x, region)))
            }
            elements::InlineElement::Comment(x) => {
                Self::from(Comment::from(Located::new(x, region)))
            }
        }
    }
}
