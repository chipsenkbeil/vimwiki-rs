use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{self as v, Located};

/// Represents a single document inline set of tags
#[gql_ent]
pub struct Tags {
    /// The segment of the document this inline set of tags covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The set of tag names
    names: Vec<String>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.names().join(":"))
    }
}

impl<'a> FromVimwikiElement<'a> for Tags {
    type Element = Located<v::Tags<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .names(
                    element
                        .into_inner()
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
    use entity_inmemory::InmemoryDatabase;
    use vimwiki::macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_tags!(r#":some:set:of:tags:"#);
            let region = Region::from(element.region());
            let ent = Tags::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(
                ent.names(),
                &vec![
                    "some".to_string(),
                    "set".to_string(),
                    "of".to_string(),
                    "tags".to_string()
                ]
            );
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
