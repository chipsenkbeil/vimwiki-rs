use crate::data::{
    ConvertToDatabaseError, InlineElement, InlineElementQuery, Region,
};

use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct Header {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    level: i32,
    centered: bool,

    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

#[async_graphql::Object]
impl Header {
    /// The segment of the document this header covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The depth of the header in the document (1 being top level, max depth of 6)
    #[graphql(name = "level")]
    async fn gql_level(&self) -> i32 {
        *self.level()
    }

    /// Whether or not the header is centered
    #[graphql(name = "centered")]
    async fn gql_centered(&self) -> bool {
        *self.centered()
    }

    /// The content within the header as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(&self) -> async_graphql::Result<Vec<InlineElement>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the header as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }
}

impl<'a> TryFrom<Located<v::Header<'a>>> for Header {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Header<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let level = le.as_inner().level as i32;
        let centered = le.as_inner().centered;

        let mut contents = Vec::new();
        for content in le.into_inner().content.elements {
            contents.push(InlineElement::try_from(content)?.id());
        }

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .level(level)
                .centered(centered)
                .contents(contents)
                .finish_and_commit(),
        )
    }
}
