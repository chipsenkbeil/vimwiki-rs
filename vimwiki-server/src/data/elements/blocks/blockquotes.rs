use crate::data::{GraphqlDatabaseError, Region};

use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Blockquote {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
    lines: Vec<String>,
}

impl<'a> TryFrom<Located<v::Blockquote<'a>>> for Blockquote {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Blockquote<'a>>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
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
        let (ent, region) =
            global::with_db(InmemoryDatabase::default(), || {
                let element = vimwiki_blockquote! {r#"
                    > First line of text
                    > Second line of text
                "#};
                let region = Region::from(element.region());
                let ent = Blockquote::try_from(element)
                    .expect("Failed to convert from element");
                (ent, region)
            });

        assert_eq!(
            ent.lines(),
            &[
                "First line of text".to_string(),
                "Second line of text".to_string()
            ],
        );
        assert_eq!(ent.region(), &region);
    }
}
