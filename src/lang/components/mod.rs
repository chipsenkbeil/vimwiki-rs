use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod blockquotes;
pub use blockquotes::*;

mod comments;
pub use comments::*;

mod dividers;
pub use dividers::*;

mod headers;
pub use headers::*;

mod links;
pub use links::*;

mod lists;
pub use lists::*;

mod math;
pub use math::*;

mod paragraphs;
pub use paragraphs::*;

mod preformatted;
pub use preformatted::*;

mod tables;
pub use tables::*;

mod tags;
pub use tags::*;

mod typefaces;
pub use typefaces::*;

/// Represents a full page containing different components
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page {
    components: Vec<BlockComponent>,
}

impl Page {
    pub fn components(&self) -> &[BlockComponent] {
        &self.components
    }
}

/// Represents components that can be dropped into other components
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum InlineComponent {
    Text(String),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    TagSequence(TagSequence),
    Math(MathInline),
}

/// Represents components that are standalone (metaphorically a block element in CSS)
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockComponent {
    Header(Header),
    Paragraph(Paragraph),
    List(List),
    Table(Table),
    PreformattedText(PreformattedText),
    Math(MathBlock),
    Blockquote(Blockquote),
    Divider(Divider),
    TagSequence(TagSequence),
}
