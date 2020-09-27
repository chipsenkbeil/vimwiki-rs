use super::{Link, Region};
use vimwiki::{components, LC};

/// Represents raw text within a single document
#[derive(async_graphql::SimpleObject)]
pub struct Text {
    /// The segment of the document this text covers
    region: Region,

    /// The text content
    content: String,
}

impl From<LC<String>> for Text {
    fn from(lc: LC<String>) -> Self {
        Self {
            region: Region::from(lc.region),
            content: lc.component,
        }
    }
}

/// Represents a typeface decoration that can be applied to text
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum Decoration {
    Bold,
    Italic,
    BoldItalic,
    Strikeout,
    Code,
    Superscript,
    Subscript,
}

impl From<components::Decoration> for Decoration {
    fn from(d: components::Decoration) -> Self {
        match d {
            components::Decoration::Bold => Self::Bold,
            components::Decoration::Italic => Self::Italic,
            components::Decoration::BoldItalic => Self::BoldItalic,
            components::Decoration::Strikeout => Self::Strikeout,
            components::Decoration::Code => Self::Code,
            components::Decoration::Superscript => Self::Superscript,
            components::Decoration::Subscript => Self::Subscript,
        }
    }
}

/// Represents content that can be contained within a decoration
#[derive(async_graphql::Union)]
pub enum DecoratedTextContent {
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
}

impl From<LC<components::DecoratedTextContent>> for DecoratedTextContent {
    fn from(lc: LC<components::DecoratedTextContent>) -> Self {
        match lc.component {
            components::DecoratedTextContent::Text(content) => {
                Self::from(Text {
                    region: Region::from(lc.region),
                    content,
                })
            }
            components::DecoratedTextContent::DecoratedText(x) => {
                Self::from(DecoratedText::from(LC::new(x, lc.region)))
            }
            components::DecoratedTextContent::Keyword(x) => {
                Self::from(Keyword {
                    region: Region::from(lc.region),
                    r#type: KeywordType::from(x),
                })
            }
            components::DecoratedTextContent::Link(x) => {
                Self::from(Link::from(LC::new(x, lc.region)))
            }
        }
    }
}

/// Represents text (series of content) with a typeface decoration
#[derive(async_graphql::SimpleObject)]
pub struct DecoratedText {
    /// The segment of the document this decorated text covers
    region: Region,

    /// The contents within the decoration
    contents: Vec<DecoratedTextContent>,

    /// The decoration applied to the contents
    decoration: Decoration,
}

impl From<LC<components::DecoratedText>> for DecoratedText {
    fn from(mut lc: LC<components::DecoratedText>) -> Self {
        Self {
            region: Region::from(lc.region),
            contents: lc
                .component
                .contents
                .drain(..)
                .map(DecoratedTextContent::from)
                .collect(),
            decoration: Decoration::from(lc.component.decoration),
        }
    }
}

/// Represents type of special keywords that have unique syntax highlighting
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum KeywordType {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}

impl From<components::Keyword> for KeywordType {
    fn from(k: components::Keyword) -> Self {
        match k {
            components::Keyword::TODO => KeywordType::TODO,
            components::Keyword::DONE => KeywordType::DONE,
            components::Keyword::STARTED => KeywordType::STARTED,
            components::Keyword::FIXME => KeywordType::FIXME,
            components::Keyword::FIXED => KeywordType::FIXED,
            components::Keyword::XXX => KeywordType::XXX,
        }
    }
}

/// Represents special keywords that have unique syntax highlighting
#[derive(async_graphql::SimpleObject)]
pub struct Keyword {
    /// The segment of the document this keyword covers
    region: Region,

    /// The type of keyword
    r#type: KeywordType,
}

impl From<LC<components::Keyword>> for Keyword {
    fn from(lc: LC<components::Keyword>) -> Self {
        Self {
            region: Region::from(lc.region),
            r#type: KeywordType::from(lc.component),
        }
    }
}
