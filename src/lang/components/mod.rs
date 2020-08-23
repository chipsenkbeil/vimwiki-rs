use super::utils::LC;
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
pub struct Page(Vec<LC<BlockComponent>>);

impl Page {
    pub fn components(&self) -> &[LC<BlockComponent>] {
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
    EmptyLine,
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
pub struct InlineComponentContainer(Vec<LC<InlineComponent>>);

impl From<LC<InlineComponent>> for InlineComponentContainer {
    fn from(component: LC<InlineComponent>) -> Self {
        Self::new(vec![component])
    }
}

impl From<LC<&str>> for InlineComponentContainer {
    fn from(component: LC<&str>) -> Self {
        Self::from(component.map(|x| x.to_string()))
    }
}

impl From<LC<String>> for InlineComponentContainer {
    fn from(component: LC<String>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}

impl From<LC<DecoratedText>> for InlineComponentContainer {
    fn from(component: LC<DecoratedText>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}

impl From<LC<Keyword>> for InlineComponentContainer {
    fn from(component: LC<Keyword>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}

impl From<LC<Link>> for InlineComponentContainer {
    fn from(component: LC<Link>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}

impl From<LC<TagSequence>> for InlineComponentContainer {
    fn from(component: LC<TagSequence>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}

impl From<LC<MathInline>> for InlineComponentContainer {
    fn from(component: LC<MathInline>) -> Self {
        Self::from(component.map(InlineComponent::from))
    }
}
