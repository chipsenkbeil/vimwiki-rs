use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
};
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
pub struct Page(Vec<BlockComponent>);

impl Page {
    pub fn components(&self) -> &[BlockComponent] {
        &self.0
    }
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

/// Represents components that can be dropped into other components
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InlineComponent {
    Text(String),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    TagSequence(TagSequence),
    Math(MathInline),
}

/// Represents a convenience wrapper around a series of inline components
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct InlineComponentContainer(Vec<InlineComponent>);

impl From<InlineComponent> for InlineComponentContainer {
    fn from(component: InlineComponent) -> Self {
        Self::new(vec![component])
    }
}

impl From<&str> for InlineComponentContainer {
    fn from(component: &str) -> Self {
        Self::from(InlineComponent::from(component.to_string()))
    }
}

impl From<String> for InlineComponentContainer {
    fn from(component: String) -> Self {
        Self::from(InlineComponent::from(component))
    }
}

impl From<DecoratedText> for InlineComponentContainer {
    fn from(component: DecoratedText) -> Self {
        Self::from(InlineComponent::from(component))
    }
}

impl From<Keyword> for InlineComponentContainer {
    fn from(component: Keyword) -> Self {
        Self::from(InlineComponent::from(component))
    }
}

impl From<Link> for InlineComponentContainer {
    fn from(component: Link) -> Self {
        Self::from(InlineComponent::from(component))
    }
}

impl From<TagSequence> for InlineComponentContainer {
    fn from(component: TagSequence) -> Self {
        Self::from(InlineComponent::from(component))
    }
}

impl From<MathInline> for InlineComponentContainer {
    fn from(component: MathInline) -> Self {
        Self::from(InlineComponent::from(component))
    }
}
