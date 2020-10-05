use super::{Program, Wiki};

pub mod elements;

/// Represents the query-portion of the GraphQL schema
pub struct Query;

#[async_graphql::Object]
impl Query {
    #[field(desc = "Returns a wiki using its index")]
    async fn wiki_at_index<'a>(
        &'a self,
        ctx: &'a async_graphql::Context<'_>,
        index: u32,
    ) -> Option<&'a Wiki> {
        ctx.data_unchecked::<Program>().wiki_by_index(index)
    }

    #[field(desc = "Returns a wiki using its name")]
    async fn wiki_with_name<'a>(
        &'a self,
        ctx: &'a async_graphql::Context<'_>,
        name: String,
    ) -> Option<&'a Wiki> {
        ctx.data_unchecked::<Program>().wiki_by_name(&name)
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
