use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

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

#[simple_ent]
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

impl<'a> TryFrom<Located<v::InlineElement<'a>>> for InlineElement {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::InlineElement<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::InlineElement::Text(x) => {
                Self::Text(Text::try_from(Located::new(x, region))?)
            }
            v::InlineElement::DecoratedText(x) => Self::DecoratedText(
                DecoratedText::try_from(Located::new(x, region))?,
            ),
            v::InlineElement::Keyword(x) => {
                Self::Keyword(Keyword::try_from(Located::new(x, region))?)
            }
            v::InlineElement::Link(x) => {
                Self::Link(Link::try_from(Located::new(x, region))?)
            }
            v::InlineElement::Tags(x) => {
                Self::Tags(Tags::try_from(Located::new(x, region))?)
            }
            v::InlineElement::Code(x) => {
                Self::Code(CodeInline::try_from(Located::new(x, region))?)
            }
            v::InlineElement::Math(x) => {
                Self::Math(MathInline::try_from(Located::new(x, region))?)
            }
            v::InlineElement::Comment(x) => {
                Self::Comment(Comment::try_from(Located::new(x, region))?)
            }
        })
    }
}
