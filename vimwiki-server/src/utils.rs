use entity::*;

#[inline]
pub fn gql_db() -> async_graphql::Result<DatabaseRc> {
    WeakDatabaseRc::upgrade(&entity::global::db())
        .ok_or(async_graphql::Error::new("Database unavailable"))
}
