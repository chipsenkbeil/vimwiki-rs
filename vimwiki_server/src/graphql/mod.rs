use vimwiki_macros::*;

pub mod elements;

/// Represents the query-portion of the GraphQL schema
pub struct Query;

#[async_graphql::Object]
impl Query {
    #[field(desc = "Returns a page")]
    async fn page(&self) -> elements::Page {
        elements::Page::from(vimwiki_page! {r#"
            = Some Header =
            =Another Header=
            =Third Header=
        "#})
    }
}

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema = async_graphql::Schema<
    Query,
    async_graphql::EmptyMutation,
    async_graphql::EmptySubscription,
>;
