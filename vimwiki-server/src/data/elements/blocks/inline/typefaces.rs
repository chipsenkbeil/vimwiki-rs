use crate::data::{ConvertToDatabaseError, Link, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

/// Represents raw text within a single document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Text {
    /// The segment of the document this text covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The text content
    content: String,
}

impl<'a> TryFrom<Located<v::Text<'a>>> for Text {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Text<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .content(le.into_inner().to_string())
                .finish_and_commit(),
        )
    }
}

/// Represents content that can be contained within a decoration
#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum DecoratedTextContent {
    Text(Text),
    Keyword(Keyword),
    #[graphql(flatten)]
    Link(Link),
    DecoratedText(DecoratedText),
}

impl<'a> TryFrom<Located<v::DecoratedTextContent<'a>>>
    for DecoratedTextContent
{
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::DecoratedTextContent<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::DecoratedTextContent::Text(x) => Self::Text(Text {
                region: Region::from(region),
                content: x.to_string(),
            }),
            v::DecoratedTextContent::DecoratedText(x) => Self::DecoratedText(
                DecoratedText::try_from(Located::new(x, region))?,
            ),
            v::DecoratedTextContent::Keyword(x) => Self::Keyword(Keyword {
                region: Region::from(region),
                r#type: KeywordType::from(x),
            }),
            v::DecoratedTextContent::Link(x) => {
                Self::DecoratedText(Link::try_from(Located::new(x, region))?)
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecoratedText(Located<v::DecoratedText<'static>>);

/// Represents some text (or series of inline content) that has a decoration
/// applied to it
#[async_graphql::Object]
impl DecoratedText {
    /// The segment of the document this header covers
    async fn region(&self) -> Region {
        Region::from(self.0.region())
    }

    /// The content within the decoration as individual elements
    async fn content_elements(&self) -> Vec<DecoratedTextContent> {
        self.0
            .as_contents()
            .iter()
            .map(|e| DecoratedTextContent::from(e.clone()))
            .collect()
    }

    /// The content within the decoration as it would be read by humans
    /// without frills
    async fn content(&self) -> String {
        self.0.to_string()
    }

    /// Represents the decoration applied to some text
    async fn decoration(&self) -> Decoration {
        match self.0.as_inner() {
            v::DecoratedText::Bold(_) => Decoration::Bold,
            v::DecoratedText::Italic(_) => Decoration::Italic,
            v::DecoratedText::Strikeout(_) => Decoration::Strikeout,
            v::DecoratedText::Superscript(_) => Decoration::Superscript,
            v::DecoratedText::Subscript(_) => Decoration::Subscript,
        }
    }
}

impl<'a> From<Located<v::DecoratedText<'a>>> for DecoratedText {
    fn from(le: Located<v::DecoratedText<'a>>) -> Self {
        let region = le.region();
        Self(Located::new(le.into_inner().into_owned(), region))
    }
}

/// Represents the type of decoration to apply to some text
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq, ValueLike)]
pub enum Decoration {
    Bold,
    Italic,
    Strikeout,
    Superscript,
    Subscript,
}

/// Represents type of special keywords that have unique syntax highlighting
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeywordType {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}

impl From<v::Keyword> for KeywordType {
    fn from(k: v::Keyword) -> Self {
        match k {
            v::Keyword::TODO => KeywordType::TODO,
            v::Keyword::DONE => KeywordType::DONE,
            v::Keyword::STARTED => KeywordType::STARTED,
            v::Keyword::FIXME => KeywordType::FIXME,
            v::Keyword::FIXED => KeywordType::FIXED,
            v::Keyword::XXX => KeywordType::XXX,
        }
    }
}

/// Represents special keywords that have unique syntax highlighting
#[simple_ent]
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Keyword {
    /// The segment of the document this keyword covers
    region: Region,

    /// The type of keyword
    r#type: KeywordType,
}

impl From<Located<v::Keyword>> for Keyword {
    fn from(le: Located<v::Keyword>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            r#type: KeywordType::from(le.into_inner()),
        }
    }
}
