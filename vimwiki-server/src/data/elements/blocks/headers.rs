use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};

use entity::*;
use entity_async_graphql::*;
use std::fmt;
use vimwiki::{self as v, Located};

#[gql_ent]
pub struct Header {
    /// The segment of the document this header covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The depth of the header in the document (1 being top level, max depth of 6)
    level: i32,

    /// Whether or not the header is centered
    centered: bool,

    /// The content within the header as individual elements
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// The content within the header as it would be read by humans
    /// without frills
    #[ent(field(computed = "self.to_string()"))]
    text: String,

    /// The page containing this header
    #[ent(edge)]
    page: Page,

    /// The parent element containing this header
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Header {
    type Element = Located<v::Header<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let level = element.as_inner().level as i32;
        let centered = element.as_inner().centered;

        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .level(level)
                .centered(centered)
                .contents(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for content in element.into_inner().content {
            contents.push(
                InlineElement::from_vimwiki_element(
                    page_id,
                    Some(ent.id()),
                    content,
                )?
                .id(),
            );
        }

        ent.set_contents_ids(contents);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
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
            let element = vimwiki_header!(r#"=== *some* header of mine ==="#);
            let region = Region::from(element.region());
            let ent = Header::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(*ent.level(), 3);
            assert_eq!(*ent.centered(), false);
            assert_eq!(ent.to_string(), "some header of mine");
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            for content in ent.load_contents().expect("Failed to load contents")
            {
                assert_eq!(content.page_id(), 999);
                assert_eq!(content.parent_id(), Some(ent.id()));
            }
        });
    }
}
