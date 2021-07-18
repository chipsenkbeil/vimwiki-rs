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
pub struct Paragraph {
    /// The segment of the document this paragraph covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The content within the paragraph as individual elements
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// The content within the paragraph as it would be read by humans
    /// without frills
    #[ent(field(computed = "self.to_string()"))]
    text: String,

    /// The page containing this paragraph
    #[ent(edge)]
    page: Page,

    /// The parent element containing this paragraph
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

impl fmt::Display for Paragraph {
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

impl<'a> FromVimwikiElement<'a> for Paragraph {
    type Element = Located<v::Paragraph<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());

        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut contents = Vec::new();
        for line in element.into_inner().lines {
            for content in line {
                contents.push(
                    InlineElement::from_vimwiki_element(
                        page_id,
                        Some(ent.id()),
                        content,
                    )?
                    .id(),
                );
            }
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
            let element = vimwiki_paragraph!(r#"some paragraph"#);
            let region = Region::from(element.region());
            let ent = Paragraph::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.to_string(), "some paragraph");
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
