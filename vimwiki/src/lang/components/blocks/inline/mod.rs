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

/// Represents components that can be dropped into other components
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InlineComponent {
    Text(String),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    Tags(Tags),
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
pub struct InlineComponentContainer {
    pub components: Vec<LC<InlineComponent>>,
}

impl From<Vec<InlineComponentContainer>> for InlineComponentContainer {
    fn from(mut containers: Vec<Self>) -> Self {
        Self::new(containers.drain(..).flat_map(|c| c.components).collect())
    }
}

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

macro_rules! container_mapping {
    ($type:ty) => {
        impl From<$type> for InlineComponentContainer {
            fn from(component: $type) -> Self {
                Self::from(component.map(InlineComponent::from))
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
