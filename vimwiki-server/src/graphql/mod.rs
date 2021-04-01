mod mutation;
mod query;

pub use mutation::Mutation;
pub use query::Query;

/// Represents the overall schema for the vimwiki GraphQL server
pub type Schema =
    async_graphql::Schema<Query, Mutation, async_graphql::EmptySubscription>;

pub fn new_schema() -> Schema {
    Schema::build(Query, Mutation, async_graphql::EmptySubscription).finish()
}
