mod blocks;
pub use blocks::*;

mod utils;
pub use utils::*;

use crate::data::GraphqlDatabaseError;
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Page {
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<BlockElement>,
}

impl<'a> TryFrom<v::Page<'a>> for Page {
    type Error = GraphqlDatabaseError;

    fn try_from(page: v::Page<'a>) -> Result<Self, Self::Error> {
        let mut contents = Vec::new();
        for content in page.elements {
            contents.push(BlockElement::try_from(content)?.id());
        }

        GraphqlDatabaseError::wrap(
            Self::build().contents(contents).finish_and_commit(),
        )
    }
}

#[simple_ent]
#[derive(async_graphql::Union)]
pub enum Element {
    #[ent(wrap)]
    #[graphql(flatten)]
    Block(BlockElement),

    #[ent(wrap)]
    #[graphql(flatten)]
    Inline(InlineElement),

    #[ent(wrap)]
    #[graphql(flatten)]
    InlineBlock(InlineBlockElement),
}

impl<'a> TryFrom<Located<v::Element<'a>>> for Element {
    type Error = GraphqlDatabaseError;

    fn try_from(located: Located<v::Element<'a>>) -> Result<Self, Self::Error> {
        let region = located.region();
        Ok(match located.into_inner() {
            v::Element::Block(x) => {
                Self::from(BlockElement::try_from(Located::new(x, region))?)
            }
            v::Element::Inline(x) => {
                Self::from(InlineElement::try_from(Located::new(x, region))?)
            }
            v::Element::InlineBlock(x) => Self::from(
                InlineBlockElement::try_from(Located::new(x, region))?,
            ),
        })
    }
}

#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum InlineBlockElement {
    ListItem(ListItem),
    Term(Term),
    Definition(Definition),
}

impl<'a> TryFrom<Located<v::InlineBlockElement<'a>>> for InlineBlockElement {
    type Error = GraphqlDatabaseError;

    fn try_from(
        located: Located<v::InlineBlockElement<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = located.region();
        Ok(match located.into_inner() {
            v::InlineBlockElement::ListItem(x) => InlineBlockElement::from(
                ListItem::try_from(Located::new(x, region))?,
            ),
            v::InlineBlockElement::Term(x) => InlineBlockElement::from(
                Term::try_from(Located::new(x, region))?,
            ),
            v::InlineBlockElement::Definition(x) => InlineBlockElement::from(
                Definition::try_from(Located::new(x, region))?,
            ),
        })
    }
}
