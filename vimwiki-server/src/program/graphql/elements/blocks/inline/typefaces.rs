use super::{Link, Region};
use vimwiki::{elements, Located};

/// Represents raw text within a single document
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Text {
    /// The segment of the document this text covers
    region: Region,

    /// The text content
    content: String,
}

impl<'a> From<Located<elements::Text<'a>>> for Text {
    fn from(le: Located<elements::Text<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            content: le.into_inner().to_string(),
        }
    }
}

/// Represents content that can be contained within a decoration
#[derive(async_graphql::Union, Debug)]
pub enum DecoratedTextContent {
    Text(Text),
    Keyword(Keyword),
    #[graphql(flatten)]
    Link(Link),
    DecoratedText(DecoratedText),
}

impl<'a> From<Located<elements::DecoratedTextContent<'a>>>
    for DecoratedTextContent
{
    fn from(le: Located<elements::DecoratedTextContent<'a>>) -> Self {
        let region = le.region();
        match le.into_inner() {
            elements::DecoratedTextContent::Text(x) => Self::from(Text {
                region: Region::from(region),
                content: x.to_string(),
            }),
            elements::DecoratedTextContent::DecoratedText(x) => {
                Self::from(DecoratedText::from(Located::new(x, region)))
            }
            elements::DecoratedTextContent::Keyword(x) => Self::from(Keyword {
                region: Region::from(region),
                r#type: KeywordType::from(x),
            }),
            elements::DecoratedTextContent::Link(x) => {
                Self::from(Link::from(Located::new(x, region)))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecoratedText(Located<elements::DecoratedText<'static>>);

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
            elements::DecoratedText::Bold(_) => Decoration::Bold,
            elements::DecoratedText::Italic(_) => Decoration::Italic,
            elements::DecoratedText::Strikeout(_) => Decoration::Strikeout,
            elements::DecoratedText::Superscript(_) => Decoration::Superscript,
            elements::DecoratedText::Subscript(_) => Decoration::Subscript,
        }
    }
}

impl<'a> From<Located<elements::DecoratedText<'a>>> for DecoratedText {
    fn from(le: Located<elements::DecoratedText<'a>>) -> Self {
        let region = le.region();
        Self(Located::new(le.into_inner().into_owned(), region))
    }
}

/// Represents the type of decoration to apply to some text
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
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

impl From<elements::Keyword> for KeywordType {
    fn from(k: elements::Keyword) -> Self {
        match k {
            elements::Keyword::TODO => KeywordType::TODO,
            elements::Keyword::DONE => KeywordType::DONE,
            elements::Keyword::STARTED => KeywordType::STARTED,
            elements::Keyword::FIXME => KeywordType::FIXME,
            elements::Keyword::FIXED => KeywordType::FIXED,
            elements::Keyword::XXX => KeywordType::XXX,
        }
    }
}

/// Represents special keywords that have unique syntax highlighting
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Keyword {
    /// The segment of the document this keyword covers
    region: Region,

    /// The type of keyword
    r#type: KeywordType,
}

impl From<Located<elements::Keyword>> for Keyword {
    fn from(le: Located<elements::Keyword>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            r#type: KeywordType::from(le.into_inner()),
        }
    }
}
