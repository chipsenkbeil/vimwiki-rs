use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};

use entity::*;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Blockquote {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
    lines: Vec<String>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for Blockquote {
    type Element = Located<v::Blockquote<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(element.region()))
                .lines(
                    element
                        .into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_blockquote! {r#"
                > First line of text
                > Second line of text
            "#};
            let region = Region::from(element.region());
            let ent = Blockquote::from_vimwiki_element(999, Some(123), element)
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
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
