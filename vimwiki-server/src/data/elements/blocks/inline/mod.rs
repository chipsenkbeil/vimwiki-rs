use crate::data::{FromVimwikiElement, GraphqlDatabaseError};
use derive_more::Display;
use entity::*;
use entity_async_graphql::*;
use vimwiki::{self as v, Located};

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

#[gql_ent]
#[derive(Debug, Display)]
pub enum InlineElement {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    #[ent(wrap)]
    #[graphql(flatten)]
    Link(Link),
    Tags(Tags),
    Code(CodeInline),
    Math(MathInline),
    #[ent(wrap)]
    #[graphql(flatten)]
    Comment(Comment),
}

impl InlineElement {
    pub fn page_id(&self) -> Id {
        match self {
            Self::Text(x) => x.page_id(),
            Self::DecoratedText(x) => x.page_id(),
            Self::Keyword(x) => x.page_id(),
            Self::Link(x) => x.page_id(),
            Self::Tags(x) => x.page_id(),
            Self::Code(x) => x.page_id(),
            Self::Math(x) => x.page_id(),
            Self::Comment(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Text(x) => x.parent_id(),
            Self::DecoratedText(x) => x.parent_id(),
            Self::Keyword(x) => x.parent_id(),
            Self::Link(x) => x.parent_id(),
            Self::Tags(x) => x.parent_id(),
            Self::Code(x) => x.parent_id(),
            Self::Math(x) => x.parent_id(),
            Self::Comment(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for InlineElement {
    type Element = Located<v::InlineElement<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::InlineElement::Text(x) => {
                Self::Text(Text::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::DecoratedText(x) => {
                Self::DecoratedText(DecoratedText::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Keyword(x) => {
                Self::Keyword(Keyword::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Link(x) => {
                Self::Link(Link::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Tags(x) => {
                Self::Tags(Tags::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Code(x) => {
                Self::Code(CodeInline::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Math(x) => {
                Self::Math(MathInline::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineElement::Comment(x) => {
                Self::Comment(Comment::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}
