mod misc;
mod obj;

pub use misc::MiscQuery;
pub use obj::ObjQuery;

/// Represents the query-portion of the GraphQL schema
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(ObjQuery, MiscQuery);
