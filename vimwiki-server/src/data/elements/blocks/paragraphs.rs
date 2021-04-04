use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};
use entity::*;
use std::fmt;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct Paragraph {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// Page containing the paragraph
    #[ent(edge)]
    page: Page,

    /// Parent element to this paragraph
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
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

#[async_graphql::Object]
impl Paragraph {
    /// The segment of the document this paragraph covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the paragraph as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(&self) -> async_graphql::Result<Vec<InlineElement>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the paragraph as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }

    /// The page containing this paragraph
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this paragraph
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
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
        for content in element.into_inner().content.elements {
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
    use vimwiki_macros::*;

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
