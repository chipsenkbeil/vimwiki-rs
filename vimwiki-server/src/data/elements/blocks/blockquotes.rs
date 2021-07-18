use crate::data::{
    elements::build_gql_element, Element, ElementQuery, FromVimwikiElement,
    FromVimwikiElementArgs, GqlPageFilter, GraphqlDatabaseError, Page,
    PageQuery, Region,
};

use entity::*;
use entity_async_graphql::*;
use vimwiki::{self as v, Located};

#[gql_ent]
pub struct Blockquote {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,
    lines: Vec<String>,

    /// Page containing the blockquote
    #[ent(edge)]
    page: Page,

    /// Root element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    root: Element,

    /// Parent element to this blockquote
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for Blockquote {
    type Element = Located<v::Blockquote<'a>>;

    fn from_vimwiki_element(
        args: FromVimwikiElementArgs<Self::Element>,
    ) -> Result<Self, GraphqlDatabaseError> {
        build_gql_element!(
            args,
            |builder: BlockquoteBuilder, element: Self::Element| {
                builder.region(Region::from(element.region())).lines(
                    element
                        .into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity_inmemory::InmemoryDatabase;
    use vimwiki::macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_blockquote! {r#"
                > First line of text
                > Second line of text
            "#};
            let region = Region::from(element.region());
            let args = FromVimwikiElementArgs {
                page_id: 999,
                root_id: 1,
                parent_id: Some(2),
                prev_sibling_id: Some(3),
                next_sibling_id: Some(4),
                element_id: 5,
                element,
            };
            let ent = Blockquote::from_vimwiki_element(args)
                .expect("Failed to convert from element");

            assert_eq!(
                ent.lines(),
                &[
                    "First line of text".to_string(),
                    "Second line of text".to_string()
                ],
            );
            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.root_id(), 1);
            assert_eq!(ent.parent_id(), Some(2));
            assert_eq!(ent.prev_sibling_id(), Some(3));
            assert_eq!(ent.next_sibling_id(), Some(4));
            assert_eq!(ent.id(), 5);
        });
    }
}
