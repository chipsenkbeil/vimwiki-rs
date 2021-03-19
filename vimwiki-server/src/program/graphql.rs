use crate::{data::Wiki, program::Program};

/// Represents the query-portion of the GraphQL schema
pub struct Query;

#[async_graphql::Object]
impl Query {
    /// Returns a wiki using its index
    async fn wiki_at_index<'a>(
        &self,
        ctx: &async_graphql::Context<'_>,
        index: u32,
    ) -> Option<Wiki> {
        ctx.data_unchecked::<Program>()
            .wiki_by_index(index as usize)
            .await
    }

    /// Returns a wiki using its name
    async fn wiki_with_name<'a>(
        &self,
        ctx: &async_graphql::Context<'_>,
        name: String,
    ) -> Option<Wiki> {
        ctx.data_unchecked::<Program>().wiki_by_name(&name).await
    }

    /// Returns all pages loaded by the server
    async fn pages(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Vec<elements::Page> {
        ctx.data_unchecked::<Program>().graphql_pages().await
    }

    /// Returns the page at the specified path
    async fn page(
        &self,
        ctx: &async_graphql::Context<'_>,
        path: String,
        #[graphql(default)] reload: bool,
    ) -> Option<elements::Page> {
        ctx.data_unchecked::<Program>()
            .load_and_watch_graphql_page(path, reload)
            .await
    }
}

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema = async_graphql::Schema<
    Query,
    async_graphql::EmptyMutation,
    async_graphql::EmptySubscription,
>;

pub fn new_schema() -> Schema {
    Schema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .finish()
}
