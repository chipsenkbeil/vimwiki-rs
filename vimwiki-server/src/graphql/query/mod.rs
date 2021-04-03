/// Provides a reference to a typed version of the GraphQL database if available
macro_rules! gql_db_typed_ref {
    () => {
        crate::database::gql_db()?
            .as_ref()
            .as_database::<entity::InmemoryDatabase>()
            .ok_or_else(|| {
                async_graphql::Error::new("Invalid database type found")
            })
    };
}

mod misc;
mod obj;

pub use misc::MiscQuery;
pub use obj::ObjQuery;

/// Represents the query-portion of the GraphQL schema
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(ObjQuery, MiscQuery);
