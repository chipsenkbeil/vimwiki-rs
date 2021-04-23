use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(EntFilter)]
pub struct MathBlock {
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    lines: Vec<String>,
    environment: Option<String>,

    /// Page containing this math block
    #[ent(edge)]
    page: Page,

    /// Parent element to this math block
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
}

/// Represents a single document multi-line math formula
#[async_graphql::Object]
impl MathBlock {
    /// The segment of the document this math block covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this math block
    #[graphql(name = "lines")]
    async fn gql_lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.lines.join(" ")
    }

    /// The environment associated with this math block
    #[graphql(name = "environment")]
    async fn gql_environment(&self) -> Option<&String> {
        self.environment.as_ref()
    }

    /// The page containing this math block
    #[graphql(name = "page")]
    async fn gql_page(&self) -> async_graphql::Result<Page> {
        self.load_page()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The parent element containing this math block
    #[graphql(name = "parent")]
    async fn gql_parent(&self) -> async_graphql::Result<Option<Element>> {
        self.load_parent()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}

impl<'a> FromVimwikiElement<'a> for MathBlock {
    type Element = Located<v::MathBlock<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let lines = element
            .as_inner()
            .lines
            .iter()
            .map(ToString::to_string)
            .collect();
        let environment = element
            .as_inner()
            .environment
            .as_ref()
            .map(ToString::to_string);

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .lines(lines)
                .environment(environment)
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
            let element = vimwiki_math_block! {r#"
                {{$%align%
                math
                }}$
            "#};
            let region = Region::from(element.region());
            let ent = MathBlock::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.lines(), &["math".to_string(),]);
            assert_eq!(ent.environment(), &Some("align".to_string()));
            assert_eq!(ent.region(), &region);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));
        });
    }
}
