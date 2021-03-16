use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::{collections::HashMap, convert::TryFrom};
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct PreformattedText {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    language: Option<String>,
    metadata: HashMap<String, String>,
    lines: Vec<String>,
}

/// Represents a single document block of preformatted text (aka code block)
#[async_graphql::Object]
impl PreformattedText {
    /// The segment of the document this preformatted text covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The lines of content contained within this preformatted text
    #[graphql(name = "lines")]
    async fn gql_lines(&self) -> &[String] {
        self.lines()
    }

    /// The lines joined with " " inbetween
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.lines().join(" ")
    }

    /// The language associated with this preformatted text
    #[graphql(name = "language")]
    async fn gql_language(&self) -> Option<String> {
        self.language()
            .as_deref()
            .or_else(|| {
                self.metadata
                    .get("class")
                    .and_then(|x| x.strip_prefix("brush:"))
            })
            .map(|x| x.trim().to_string())
    }

    /// The metadata associated with some key
    #[graphql(name = "metadata_for_key")]
    async fn gql_metadata_for_key(&self, key: String) -> Option<&String> {
        self.metadata().get(&key)
    }

    /// All metadata associated with the preformatted text
    #[graphql(name = "metadata")]
    async fn gql_metadata(&self) -> &HashMap<String, String> {
        self.metadata()
    }
}

impl<'a> TryFrom<Located<v::PreformattedText<'a>>> for PreformattedText {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::PreformattedText<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let language = le.as_inner().lang.as_ref().map(ToString::to_string);
        let lines = le
            .as_inner()
            .lines
            .iter()
            .map(ToString::to_string)
            .collect();
        let metadata = le
            .into_inner()
            .metadata
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .language(language)
                .lines(lines)
                .metadata(metadata)
                .finish_and_commit(),
        )
    }
}
