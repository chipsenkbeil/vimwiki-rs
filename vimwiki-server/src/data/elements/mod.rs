mod blocks;
pub use blocks::*;

mod utils;
pub use utils::*;

use derive_more::Display;
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[derive(Debug, Display)]
pub enum ConvertToDatabaseError {
    Database(DatabaseError),
    Builder(Box<dyn std::error::Error>),
}

impl ConvertToDatabaseError {
    pub fn wrap<T, E: std::error::Error + 'static>(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<T, Self> {
        match Self::try_from(x) {
            Ok(x) => Err(x),
            Err(x) => Ok(x),
        }
    }
}

impl std::error::Error for ConvertToDatabaseError {}

impl<T, E> TryFrom<Result<Result<T, DatabaseError>, E>>
    for ConvertToDatabaseError
where
    E: std::error::Error + 'static,
{
    type Error = T;

    fn try_from(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<Self, Self::Error> {
        match x {
            Ok(x) => match x {
                Ok(x) => Err(x),
                Err(x) => Ok(ConvertToDatabaseError::Database(x)),
            },
            Err(x) => Ok(ConvertToDatabaseError::Builder(Box::new(x))),
        }
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Page {
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    elements: Vec<BlockElement>,
}

#[simple_ent]
#[derive(async_graphql::Union)]
pub enum Element {
    #[graphql(flatten)]
    Block(BlockElement),

    #[graphql(flatten)]
    Inline(InlineElement),

    #[graphql(flatten)]
    InlineBlock(InlineBlockElement),
}

impl<'a> TryFrom<Located<v::Element<'a>>> for Element {
    type Error = ConvertToDatabaseError;

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
    type Error = ConvertToDatabaseError;

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
