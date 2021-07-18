/// Builds a graphql element by consuming `FromVimwikiElementArgs` and a
/// function that takes the the gql builder, the vimwiki element, and a function
/// that can be called to produce new arguments for children (takes an element),
/// returning the updated builder
macro_rules! build_gql_element {
    ($args:ident, $f:expr) => {{
        let crate::data::FromVimwikiElementArgs {
            page_id,
            root_id,
            parent_id,
            prev_sibling_id,
            next_sibling_id,
            element_id,
            element,
        } = $args;

        let builder = Self::build()
            .page(page_id)
            .root(root_id)
            .parent(parent_id)
            .prev_sibling(prev_sibling_id)
            .next_sibling(next_sibling_id)
            .id(element_id);

        #[inline]
        fn make_args<T>(element: T) -> crate::data::FromVimwikiElementArgs<T> {
            crate::data::FromVimwikiElementArgs {
                page_id,
                root_id,
                parent_id: element_id,
                prev_sibling_id,
                next_sibling_id,
                element_id,
                element,
            }
        }

        crate::data::GraphqlDatabaseError::wrap(
            $f(builder, element, make_args).finish_and_commit(),
        )
    }};
}
pub(crate) use build_gql_element;

mod blocks;
pub use blocks::*;

mod utils;
pub use utils::*;

use crate::{
    data::{
        GqlParsedFileFilter, GraphqlDatabaseError, ParsedFile, ParsedFileQuery,
    },
    database,
};
use entity::*;
use entity_async_graphql::*;
use std::iter;
use vimwiki::{self as v, Located};

#[gql_ent]
pub struct Page {
    #[ent(edge)]
    file: ParsedFile,

    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<BlockElement>,
}

impl Page {
    pub fn create_from_vimwiki(
        file_id: Id,
        page: v::Page<'_>,
    ) -> Result<Self, GraphqlDatabaseError> {
        // First, create the page and element ids
        let page_id = database::next_id();

        // Second, pre-allocate the children ids
        let children_ids = iter::repeat_with(|| database::next_id())
            .take(page.elements.len())
            .collect::<Vec<Id>>();

        // Third, build the actual children
        for (idx, element) in page.into_elements().into_iter().enumerate() {
            let element_id = children_ids[idx];
            let args = FromVimwikiElementArgs {
                page_id,
                root_id: element_id,
                parent_id: None,
                prev_sibling_id: if idx > 0 {
                    Some(children_ids[idx])
                } else {
                    None
                },
                next_sibling_id: children_ids.get(idx + 1).copied(),
                element_id,
                element,
            };

            children_ids.push(BlockElement::from_vimwiki_element(args)?.id());
        }

        // Fourth, build the page
        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .id(page_id)
                .file(file_id)
                .contents(children_ids)
                .finish_and_commit(),
        )?;

        Ok(ent)
    }
}

/// Arguments to provide when building a new element entity
#[derive(Clone, Debug)]
pub(crate) struct FromVimwikiElementArgs<T> {
    pub page_id: Id,
    pub root_id: Id,
    pub parent_id: Option<Id>,
    pub prev_sibling_id: Option<Id>,
    pub next_sibling_id: Option<Id>,
    pub element_id: Id,
    pub element: T,
}

/// Interface to build entity from a vimwiki element
pub(crate) trait FromVimwikiElement<'a>: Sized {
    type Element;

    fn from_vimwiki_element(
        args: FromVimwikiElementArgs<Self::Element>,
    ) -> Result<Self, GraphqlDatabaseError>;
}

#[gql_ent]
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
        args: FromVimwikiElementArgs<Self::Element>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let FromVimwikiElementArgs {
            page_id,
            root_id,
            parent_id,
            prev_sibling_id,
            next_sibling_id,
            element_id,
            element,
        } = args;

        let region = element.region();
        Ok(match element.into_inner() {
            v::Element::Block(x) => Self::from(
                BlockElement::from_vimwiki_element(FromVimwikiElementArgs {
                    page_id,
                    root_id,
                    parent_id,
                    prev_sibling_id,
                    next_sibling_id,
                    element_id,
                    element: Located::new(x, region),
                })?,
            ),
            v::Element::Inline(x) => Self::from(
                InlineElement::from_vimwiki_element(FromVimwikiElementArgs {
                    page_id,
                    root_id,
                    parent_id,
                    prev_sibling_id,
                    next_sibling_id,
                    element_id,
                    element: Located::new(x, region),
                })?,
            ),
            v::Element::InlineBlock(x) => {
                Self::from(InlineBlockElement::from_vimwiki_element(
                    FromVimwikiElementArgs {
                        page_id,
                        root_id,
                        parent_id,
                        prev_sibling_id,
                        next_sibling_id,
                        element_id,
                        element: Located::new(x, region),
                    },
                )?)
            }
        })
    }
}

#[gql_ent]
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
        args: FromVimwikiElementArgs<Self::Element>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let FromVimwikiElementArgs {
            page_id,
            root_id,
            parent_id,
            prev_sibling_id,
            next_sibling_id,
            element_id,
            element,
        } = args;

        let region = element.region();
        Ok(match element.into_inner() {
            v::InlineBlockElement::ListItem(x) => InlineBlockElement::from(
                ListItem::from_vimwiki_element(FromVimwikiElementArgs {
                    page_id,
                    root_id,
                    parent_id,
                    prev_sibling_id,
                    next_sibling_id,
                    element_id,
                    element: Located::new(x, region),
                })?,
            ),
            v::InlineBlockElement::Term(x) => InlineBlockElement::from(
                Term::from_vimwiki_element(FromVimwikiElementArgs {
                    page_id,
                    root_id,
                    parent_id,
                    prev_sibling_id,
                    next_sibling_id,
                    element_id,
                    element: Located::new(x, region),
                })?,
            ),
            v::InlineBlockElement::Definition(x) => InlineBlockElement::from(
                Definition::from_vimwiki_element(FromVimwikiElementArgs {
                    page_id,
                    root_id,
                    parent_id,
                    prev_sibling_id,
                    next_sibling_id,
                    element_id,
                    element: Located::new(x, region),
                })?,
            ),
        })
    }
}
