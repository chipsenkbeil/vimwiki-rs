use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, Page, PageQuery, Region,
};
use entity::*;
use entity_async_graphql::*;
use vimwiki::{elements as v, Located};

#[gql_ent]
pub struct MathBlock {
    /// The segment of the document this math block covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The lines of content contained within this math block
    lines: Vec<String>,

    /// The lines joined with " " inbetween
    #[ent(field(computed = "self.lines.join(\" \")"))]
    text: String,

    /// The environment associated with this math block
    environment: Option<String>,

    /// The page containing this math block
    #[ent(edge)]
    page: Page,

    /// The parent element containing this math block
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,
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
