use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct MathBlock {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    lines: Vec<String>,
    environment: Option<String>,
}

/// Represents a single document multi-line math formula
#[async_graphql::Object]
impl MathBlock {
    /// The segment of the document this math block covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        &self.region
    }

    /// The lines of content contained within this math block
    #[graphql(name = "lines")]
    async fn gql_lines(&self) -> &[String] {
        &self.lines
    }

    /// The lines joined with " " inbetween
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.lines.join(" ")
    }

    /// The environment associated with this math block
    #[graphql(name = "environment")]
    async fn gql_environment(&self) -> Option<&String> {
        self.environment.as_ref()
    }
}

impl<'a> TryFrom<Located<v::MathBlock<'a>>> for MathBlock {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::MathBlock<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let lines = le
            .as_inner()
            .lines
            .iter()
            .map(ToString::to_string)
            .collect();
        let environment =
            le.as_inner().environment.as_ref().map(ToString::to_string);

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .lines(lines)
                .environment(environment)
                .finish_and_commit(),
        )
    }
}
