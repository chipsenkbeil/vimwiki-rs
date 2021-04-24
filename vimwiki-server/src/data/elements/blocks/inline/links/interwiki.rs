use crate::data::{
    Anchor, Description, Element, ElementQuery, FromVimwikiElement,
    GqlPageFilter, GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{elements as v, Located};

/// Represents a single document wiki link within another wiki
/// referenced by index
#[gql_ent]
pub struct IndexedInterWikiLink {
    /// The segment of the document this link covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The index of the wiki this link is associated with
    index: i32,

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

impl fmt::Display for IndexedInterWikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for IndexedInterWikiLink {
    type Element = Located<v::IndexedInterWikiLink<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let element = element.into_inner();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .index(element.index as i32)
                .is_dir(element.link.is_path_dir())
                .is_local_anchor(element.link.is_local_anchor())
                .path(element.link.path.to_string_lossy().to_string())
                .description(element.link.description.map(Description::from))
                .anchor(element.link.anchor.map(Anchor::from))
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by name
#[gql_ent]
pub struct NamedInterWikiLink {
    /// The segment of the document this link covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The name of the wiki this link is associated with
    name: String,

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

impl fmt::Display for NamedInterWikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for NamedInterWikiLink {
    type Element = Located<v::NamedInterWikiLink<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let element = element.into_inner();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .name(element.name.to_string())
                .is_dir(element.link.is_path_dir())
                .is_local_anchor(element.link.is_local_anchor())
                .path(element.link.path.to_string_lossy().to_string())
                .description(element.link.description.map(Description::from))
                .anchor(element.link.anchor.map(Anchor::from))
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
    fn indexed_interwiki_link_should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_inter_wiki_link!(
                r#"[[wiki1:Link Path#one#two|Some description]]"#
            );
            let region = Region::from(element.region());
            let element = {
                let region = element.region();
                match element.into_inner() {
                    v::InterWikiLink::Indexed(x) => Located::new(x, region),
                    x => panic!("Got wrong link: {:?}", x),
                }
            };
            let ent = IndexedInterWikiLink::from_vimwiki_element(
                999,
                Some(123),
                element,
            )
            .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.path(), "Link Path");
            assert_eq!(
                ent.description(),
                &Some(Description::Text(String::from("Some description")))
            );
            assert_eq!(ent.anchor(), &Some(Anchor::new(vec!["one", "two"])));
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }

    #[test]
    fn named_interwiki_link_should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_inter_wiki_link!(
                r#"[[wn.Some Name:Link Path#one#two|Some description]]"#
            );
            let region = Region::from(element.region());
            let element = {
                let region = element.region();
                match element.into_inner() {
                    v::InterWikiLink::Named(x) => Located::new(x, region),
                    x => panic!("Got wrong link: {:?}", x),
                }
            };
            let ent = NamedInterWikiLink::from_vimwiki_element(
                999,
                Some(123),
                element,
            )
            .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.path(), "Link Path");
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
