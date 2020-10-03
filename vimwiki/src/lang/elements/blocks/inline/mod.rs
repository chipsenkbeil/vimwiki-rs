use super::*;
use derive_more::{
    Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::fmt;

mod links;
pub use links::*;
mod math;
pub use math::*;
mod tags;
pub use tags::*;
mod typefaces;
pub use typefaces::*;

/// Represents elements that can be dropped into other elements
#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
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
    pub elements: Vec<LE<InlineElement>>,
}

impl fmt::Display for InlineElementContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for le in self.elements.iter() {
            write!(f, "{}", le.element.to_string())?;
        }
        Ok(())
    }
}

impl From<Vec<InlineElementContainer>> for InlineElementContainer {
    fn from(mut containers: Vec<Self>) -> Self {
        Self::new(containers.drain(..).flat_map(|c| c.elements).collect())
    }
}

impl From<LE<InlineElement>> for InlineElementContainer {
    fn from(element: LE<InlineElement>) -> Self {
        Self::new(vec![element])
    }
}

impl From<LE<&str>> for InlineElementContainer {
    fn from(element: LE<&str>) -> Self {
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

container_mapping!(LE<MathInline>);
container_mapping!(LE<String>);
container_mapping!(LE<DecoratedText>);
container_mapping!(LE<Keyword>);
container_mapping!(LE<Link>);
container_mapping!(LE<Tags>);
