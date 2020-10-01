use super::*;
use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};

mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

/// Represents elements that can be dropped into other elements
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InlineElement {
    Text(String),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    Tags(Tags),
    Math(MathInline),
}

/// Represents a convenience wrapper around a series of inline elements
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
pub struct InlineElementContainer {
    pub elements: Vec<LC<InlineElement>>,
}

impl From<Vec<InlineElementContainer>> for InlineElementContainer {
    fn from(mut containers: Vec<Self>) -> Self {
        Self::new(containers.drain(..).flat_map(|c| c.elements).collect())
    }
}

impl From<LC<InlineElement>> for InlineElementContainer {
    fn from(element: LC<InlineElement>) -> Self {
        Self::new(vec![element])
    }
}

impl From<LC<&str>> for InlineElementContainer {
    fn from(element: LC<&str>) -> Self {
        Self::from(element.map(|x| x.to_string()))
    }
}

macro_rules! container_mapping {
    ($type:ty) => {
        impl From<$type> for InlineElementContainer {
            fn from(element: $type) -> Self {
                Self::from(element.map(InlineElement::from))
            }
        }
    };
}

container_mapping!(LC<MathInline>);
container_mapping!(LC<String>);
container_mapping!(LC<DecoratedText>);
container_mapping!(LC<Keyword>);
container_mapping!(LC<Link>);
container_mapping!(LC<Tags>);
