use crate::data::{GraphqlDatabaseError, Keyword, Link, Region, Text};
use derive_more::Display;
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEntFilter)]
pub struct DecoratedText {
    /// The segment of the document this decorated text covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The decoration applied to this decorated text
    #[ent(field, ext(async_graphql(filter_untyped)))]
    decoration: Decoration,

    /// The contents of this decorated text
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<DecoratedTextContent>,
}

impl fmt::Display for DecoratedText {
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

/// Represents some text (or series of inline content) that has a decoration
/// applied to it
#[async_graphql::Object]
impl DecoratedText {
    /// The segment of the document this decorated text is within
    #[graphql(name = "region")]
    async fn gql_region(&self) -> &Region {
        self.region()
    }

    /// The content within the decoration as individual elements
    #[graphql(name = "contents")]
    async fn gql_contents(
        &self,
    ) -> async_graphql::Result<Vec<DecoratedTextContent>> {
        self.load_contents()
            .map_err(|x| async_graphql::Error::new(x.to_string()))
    }

    /// The content within the decoration as it would be read by humans
    /// without frills
    #[graphql(name = "text")]
    async fn gql_text(&self) -> String {
        self.to_string()
    }

    /// Represents the decoration applied to some text
    #[graphql(name = "decoration")]
    async fn gql_decoration(&self) -> &Decoration {
        self.decoration()
    }
}

impl<'a> TryFrom<Located<v::DecoratedText<'a>>> for DecoratedText {
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::DecoratedText<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());

        // First, figure out the type of decoration
        let decoration = match le.as_inner() {
            v::DecoratedText::Bold(_) => Decoration::Bold,
            v::DecoratedText::Italic(_) => Decoration::Italic,
            v::DecoratedText::Strikeout(_) => Decoration::Strikeout,
            v::DecoratedText::Superscript(_) => Decoration::Superscript,
            v::DecoratedText::Subscript(_) => Decoration::Subscript,
        };

        // Second, we need to create all of the content contained within the text
        let mut contents = Vec::new();
        for content in le.into_inner().into_contents() {
            contents.push(DecoratedTextContent::try_from(content)?.id());
        }

        // Third, we create the container of the content
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .decoration(decoration)
                .contents(contents)
                .finish_and_commit(),
        )
    }
}

/// Represents content that can be contained within a decoration
#[simple_ent]
#[derive(async_graphql::Union, Debug, Display)]
pub enum DecoratedTextContent {
    Text(Text),
    Keyword(Keyword),
    #[ent(wrap)]
    #[graphql(flatten)]
    Link(Link),
    DecoratedText(DecoratedText),
}

impl<'a> TryFrom<Located<v::DecoratedTextContent<'a>>>
    for DecoratedTextContent
{
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::DecoratedTextContent<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::DecoratedTextContent::Text(x) => {
                Self::Text(Text::try_from(Located::new(x, region))?)
            }
            v::DecoratedTextContent::DecoratedText(x) => Self::DecoratedText(
                DecoratedText::try_from(Located::new(x, region))?,
            ),
            v::DecoratedTextContent::Keyword(x) => {
                Self::Keyword(Keyword::try_from(Located::new(x, region))?)
            }
            v::DecoratedTextContent::Link(x) => {
                Self::Link(Link::try_from(Located::new(x, region))?)
            }
        })
    }
}

/// Represents the type of decoration to apply to some text
#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub enum Decoration {
    Bold,
    Italic,
    Strikeout,
    Superscript,
    Subscript,
}

impl ValueLike for Decoration {
    fn into_value(self) -> Value {
        match self {
            Self::Bold => Value::from("bold"),
            Self::Italic => Value::from("italic"),
            Self::Strikeout => Value::from("strikeout"),
            Self::Superscript => Value::from("superscript"),
            Self::Subscript => Value::from("subscript"),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => match x.as_str() {
                "bold" => Ok(Self::Bold),
                "italic" => Ok(Self::Italic),
                "strikeout" => Ok(Self::Strikeout),
                "superscript" => Ok(Self::Superscript),
                "subscript" => Ok(Self::Subscript),
                _ => Err(Value::Text(x)),
            },
            x => Err(x),
        }
    }
}
