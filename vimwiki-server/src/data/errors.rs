use derive_more::Display;
use entity::*;
use std::convert::TryFrom;

#[derive(Debug, Display)]
pub enum GraphqlDatabaseError {
    Database(DatabaseError),
    Builder(Box<dyn std::error::Error>),
}

impl GraphqlDatabaseError {
    pub fn wrap<T, E: std::error::Error + 'static>(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<T, Self> {
        match Self::try_from(x) {
            Ok(x) => Err(x),
            Err(x) => Ok(x),
        }
    }
}

impl std::error::Error for GraphqlDatabaseError {}

impl<T, E> TryFrom<Result<Result<T, DatabaseError>, E>> for GraphqlDatabaseError
where
    E: std::error::Error + 'static,
{
    type Error = T;

    fn try_from(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<Self, Self::Error> {
        match x {
            Ok(x) => match x {
                Ok(x) => Err(x),
                Err(x) => Ok(GraphqlDatabaseError::Database(x)),
            },
            Err(x) => Ok(GraphqlDatabaseError::Builder(Box::new(x))),
        }
    }
}
