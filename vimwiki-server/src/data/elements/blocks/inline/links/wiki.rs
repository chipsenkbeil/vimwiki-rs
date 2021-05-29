use crate::data::{
    Anchor, Description, Element, ElementQuery, FromVimwikiElement,
    GqlPageFilter, GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{self as v, Located};

/// Represents a single document wiki link
#[gql_ent]
pub struct WikiLink {
    /// The segment of the document this link covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    #[ent(field(graphql(filter_untyped)))]
    description: Option<Description>,

    /// Optional anchor associated with the link
    #[ent(field(graphql(filter_untyped)))]
    anchor: Option<Anchor>,

    /// Page containing the element
    #[ent(edge)]
    page: Page,

    /// Parent element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for WikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for WikiLink {
    type Element = Located<v::Link<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let link = element.into_inner();
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .is_dir(link.data().is_path_dir())
                .is_local_anchor(link.data().is_local_anchor())
                .path(link.data().to_path_buf().to_string_lossy().to_string())
                .anchor(link.to_anchor().map(Anchor::from))
                .description(link.into_description().map(Description::from))
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
            let link =
                vimwiki_link!(r#"[[some path#one#two|Some description]]"#);
            let region = Region::from(link.region());
            let ent = WikiLink::from_vimwiki_element(999, Some(123), link)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.path(), "some%20path");
            assert_eq!(
                ent.description(),
                &Some(Description::Text(String::from("Some description")))
            );
            assert_eq!(ent.anchor(), &Some(Anchor::new(vec!["one", "two"])));
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
