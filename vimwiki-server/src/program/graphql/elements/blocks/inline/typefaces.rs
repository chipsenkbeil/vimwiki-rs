use super::{Link, Region};
use vimwiki::{elements, LE};

/// Represents raw text within a single document
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Text {
    /// The segment of the document this text covers
    region: Region,

    /// The text content
    content: String,
}

impl From<LE<String>> for Text {
    fn from(lc: LE<String>) -> Self {
        Self {
            region: Region::from(lc.region),
            content: lc.element,
        }
    }
}

/// Represents a typeface decoration that can be applied to text
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Decoration {
    Bold,
    Italic,
    BoldItalic,
    Strikeout,
    Code,
    Superscript,
    Subscript,
}

impl From<elements::Decoration> for Decoration {
    fn from(d: elements::Decoration) -> Self {
        match d {
            elements::Decoration::Bold => Self::Bold,
            elements::Decoration::Italic => Self::Italic,
            elements::Decoration::BoldItalic => Self::BoldItalic,
            elements::Decoration::Strikeout => Self::Strikeout,
            elements::Decoration::Code => Self::Code,
            elements::Decoration::Superscript => Self::Superscript,
            elements::Decoration::Subscript => Self::Subscript,
        }
    }
}

/// Represents content that can be contained within a decoration
#[derive(async_graphql::Union, Debug)]
pub enum DecoratedTextContent {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    #[item(flatten)]
    Link(Link),
}

impl From<LE<elements::DecoratedTextContent>> for DecoratedTextContent {
    fn from(lc: LE<elements::DecoratedTextContent>) -> Self {
        match lc.element {
            elements::DecoratedTextContent::Text(content) => Self::from(Text {
                region: Region::from(lc.region),
                content,
            }),
            elements::DecoratedTextContent::DecoratedText(x) => {
                Self::from(DecoratedText::from(LE::new(x, lc.region)))
            }
            elements::DecoratedTextContent::Keyword(x) => Self::from(Keyword {
                region: Region::from(lc.region),
                r#type: KeywordType::from(x),
            }),
            elements::DecoratedTextContent::Link(x) => {
                Self::from(Link::from(LE::new(x, lc.region)))
            }
        }
    }
}

/// Represents text (series of content) with a typeface decoration
#[derive(async_graphql::SimpleObject, Debug)]
pub struct DecoratedText {
    /// The segment of the document this decorated text covers
    region: Region,

    /// The contents within the decoration
    contents: Vec<DecoratedTextContent>,

    /// The decoration applied to the contents
    decoration: Decoration,
}

impl From<LE<elements::DecoratedText>> for DecoratedText {
    fn from(mut lc: LE<elements::DecoratedText>) -> Self {
        Self {
            region: Region::from(lc.region),
            contents: lc
                .element
                .contents
                .drain(..)
                .map(DecoratedTextContent::from)
                .collect(),
            decoration: Decoration::from(lc.element.decoration),
        }
    }
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

impl From<LE<elements::Keyword>> for Keyword {
    fn from(lc: LE<elements::Keyword>) -> Self {
        Self {
            region: Region::from(lc.region),
            r#type: KeywordType::from(lc.element),
        }
    }
}
