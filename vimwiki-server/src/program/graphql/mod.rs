use super::{Program, Wiki};

pub mod elements;

/// Represents the query-portion of the GraphQL schema
pub struct Query;

#[async_graphql::Object]
impl Query {
    /// Returns a wiki using its index
    async fn wiki_at_index<'a>(
        &'a self,
        ctx: &'a async_graphql::Context<'_>,
        index: u32,
    ) -> Option<&'a Wiki> {
        ctx.data_unchecked::<Program>()
            .wiki_by_index(index as usize)
    }

    /// Returns a wiki using its name
    async fn wiki_with_name<'a>(
        &'a self,
        ctx: &'a async_graphql::Context<'_>,
        name: String,
    ) -> Option<&'a Wiki> {
        ctx.data_unchecked::<Program>().wiki_by_name(&name)
    }

    /// Returns all pages loaded by the server
    async fn pages(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> Vec<elements::Page> {
        ctx.data_unchecked::<Program>().graphql_pages()
    }

    /// Returns the page at the specified path
    async fn page(
        &self,
        ctx: &async_graphql::Context<'_>,
        path: String,
        #[graphql(default)] _reload: bool,
    ) -> Option<elements::Page> {
        let program = ctx.data_unchecked::<Program>();
        // TODO: Need to either have some way to get a &mut of program
        //       or we have to refactor to have an async mutex
        // if reload {
        //     program.load_file(&path);
        // }
        program.graphql_page(path)
    }
}

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema = async_graphql::Schema<
    Query,
    async_graphql::EmptyMutation,
    async_graphql::EmptySubscription,
>;

pub fn build_schema_with_program(program: Program) -> Schema {
    Schema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(program)
    .finish()
}
