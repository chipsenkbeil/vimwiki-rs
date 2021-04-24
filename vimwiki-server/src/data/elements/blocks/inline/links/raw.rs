use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region, Uri,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document link formed from a raw URI
#[gql_ent]
pub struct RawLink {
    /// The segment of the document this link covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The URI representing the link
    #[ent(field(graphql(filter_untyped)))]
    uri: Uri,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for RawLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri())
    }
}

impl<'a> FromVimwikiElement<'a> for RawLink {
    type Element = Located<v::RawLink<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .uri(Uri::from(element.into_inner().uri))
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
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_raw_link!(r#"https://example.com"#);
            let region = Region::from(element.region());
            let ent = RawLink::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(
                ent.uri(),
                &"https://example.com".parse::<Uri>().unwrap()
            );
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
