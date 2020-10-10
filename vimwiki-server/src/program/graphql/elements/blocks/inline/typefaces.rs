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

impl From<LE<elements::Text>> for Text {
    fn from(le: LE<elements::Text>) -> Self {
        Self {
            region: Region::from(le.region),
            content: le.element.into(),
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
}

impl From<LE<elements::DecoratedTextContent>> for DecoratedTextContent {
    fn from(le: LE<elements::DecoratedTextContent>) -> Self {
        match le.element {
            elements::DecoratedTextContent::Text(x) => Self::from(Text {
                region: Region::from(le.region),
                content: x.into_typed().into(),
            }),
            elements::DecoratedTextContent::Keyword(x) => Self::from(Keyword {
                region: Region::from(le.region),
                r#type: KeywordType::from(x.into_typed()),
            }),
            elements::DecoratedTextContent::Link(x) => {
                Self::from(Link::from(LE::new(x.into_typed(), le.region)))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecoratedText(LE<elements::DecoratedText>);

/// Represents some text (or series of inline content) that has a decoration
/// applied to it
#[async_graphql::Object]
impl DecoratedText {
    /// The segment of the document this header covers
    async fn region(&self) -> Region {
        Region::from(self.0.region)
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
        match self.0.element {
            elements::DecoratedText::Bold(_) => Decoration::Bold,
            elements::DecoratedText::Italic(_) => Decoration::Italic,
            elements::DecoratedText::BoldItalic(_) => Decoration::BoldItalic,
            elements::DecoratedText::Strikeout(_) => Decoration::Strikeout,
            elements::DecoratedText::Superscript(_) => Decoration::Superscript,
            elements::DecoratedText::Subscript(_) => Decoration::Subscript,
        }
    }
}

impl From<LE<elements::DecoratedText>> for DecoratedText {
    fn from(le: LE<elements::DecoratedText>) -> Self {
        Self(le)
    }
}

/// Represents the type of decoration to apply to some text
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Decoration {
    Bold,
    Italic,
    BoldItalic,
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

impl From<LE<elements::Keyword>> for Keyword {
    fn from(le: LE<elements::Keyword>) -> Self {
        Self {
            region: Region::from(le.region),
            r#type: KeywordType::from(le.element),
        }
    }
}
