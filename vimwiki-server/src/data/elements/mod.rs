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
        let mut ent = GraphqlDatabaseError::wrap(
            Self::build().contents(Vec::new()).finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in page.into_elements() {
            contents.push(
                BlockElement::from_vimwiki_element(ent.id(), None, content)?
                    .id(),
            );
        }

        ent.set_contents_ids(contents);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
    }
}

/// Interface to build entity from a vimwiki element
pub trait FromVimwikiElement<'a>: Sized {
    type Element;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError>;
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

impl Element {
    pub fn page_id(&self) -> Id {
        match self {
            Self::Block(x) => x.page_id(),
            Self::Inline(x) => x.page_id(),
            Self::InlineBlock(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Block(x) => x.parent_id(),
            Self::Inline(x) => x.parent_id(),
            Self::InlineBlock(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Element {
    type Element = Located<v::Element<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::Element::Block(x) => {
                Self::from(BlockElement::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::Element::Inline(x) => {
                Self::from(InlineElement::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::Element::InlineBlock(x) => {
                Self::from(InlineBlockElement::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
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

impl InlineBlockElement {
    pub fn page_id(&self) -> Id {
        match self {
            Self::ListItem(x) => x.page_id(),
            Self::Term(x) => x.page_id(),
            Self::Definition(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::ListItem(x) => x.parent_id(),
            Self::Term(x) => x.parent_id(),
            Self::Definition(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for InlineBlockElement {
    type Element = Located<v::InlineBlockElement<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::InlineBlockElement::ListItem(x) => {
                InlineBlockElement::from(ListItem::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineBlockElement::Term(x) => {
                InlineBlockElement::from(Term::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::InlineBlockElement::Definition(x) => {
                InlineBlockElement::from(Definition::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}
