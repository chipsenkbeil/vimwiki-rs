use super::{ShareableProgram, Wiki};

pub mod elements;

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
        ctx.data_unchecked::<ShareableProgram>()
            .lock()
            .await
            .wiki_by_index(index as usize)
            .cloned()
    }

    /// Returns a wiki using its name
    async fn wiki_with_name<'a>(
        &self,
        ctx: &async_graphql::Context<'_>,
        name: String,
    ) -> Option<Wiki> {
        ctx.data_unchecked::<ShareableProgram>()
            .lock()
            .await
            .wiki_by_name(&name)
            .cloned()
    }

    /// Returns all pages loaded by the server
    async fn pages(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Vec<elements::Page> {
        ctx.data_unchecked::<ShareableProgram>()
            .lock()
            .await
            .graphql_pages()
    }

    /// Returns the page at the specified path
    async fn page(
        &self,
        ctx: &async_graphql::Context<'_>,
        path: String,
        #[graphql(default)] reload: bool,
    ) -> Option<elements::Page> {
        let mut program = ctx.data_unchecked::<ShareableProgram>().lock().await;
        if reload {
            program
                .load_file(&path)
                .await
                .ok()
                .and_then(|_| program.graphql_page(path))
        } else {
            program.graphql_page(path)
        }
    }
}

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema = async_graphql::Schema<
    Query,
    async_graphql::EmptyMutation,
    async_graphql::EmptySubscription,
>;

/// Construct our schema with the provided program as context data
pub fn build_schema_with_program(program: ShareableProgram) -> Schema {
    Schema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(program)
    .finish()
}
