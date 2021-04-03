use crate::data::Element;
use entity::{TypedPredicate as P, *};

#[derive(Default)]
pub struct MiscQuery;

#[async_graphql::Object]
impl MiscQuery {
    /// Searches for an returns the deepest element found at the given byte offset
    /// from the start of the file
    async fn element_at_offset(
        &self,
        offset: usize,
    ) -> async_graphql::Result<Option<Element>> {
        // TODO: Embed into page object instead
        // TODO: Add entity enum query filter by edge where it takes an id,
        //       optional id, or series of ids
        Element::query()
            .where_field(
                "region",
                P::has_key_where_value(
                    "offset",
                    P::and(vec![
                        P::greater_than_or_equals(offset),
                        P::less_than_or_equals(offset),
                    ]),
                )
                .into(),
            )
            .execute(gql_db_typed_ref!()?)
            .map(|x| x.into_iter().next())
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }
}
