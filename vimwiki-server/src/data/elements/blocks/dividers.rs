use crate::data::{GraphqlDatabaseError, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Divider {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
}

impl TryFrom<Located<v::Divider>> for Divider {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Divider>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .finish_and_commit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_divider!("----");
            let region = Region::from(element.region());
            let ent = Divider::try_from(element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
        });
    }
}
