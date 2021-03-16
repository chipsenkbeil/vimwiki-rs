use crate::data::{
    ConvertToDatabaseError, InlineElement, InlineElementQuery, Region,
};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct Paragraph {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,
}

impl fmt::Display for Paragraph {
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
impl Paragraph {
    /// The segment of the document this paragraph covers
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the paragraph as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(&self) -> async_graphql::Result<Vec<InlineElement>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the paragraph as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }
}

impl<'a> TryFrom<Located<v::Paragraph<'a>>> for Paragraph {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Paragraph<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        let mut contents = Vec::new();
        for content in le.into_inner().content.elements {
            contents.push(InlineElement::try_from(content)?.id());
        }

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .contents(contents)
                .finish_and_commit(),
        )
    }
}
