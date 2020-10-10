use super::*;
use derive_more::{
    Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator,
};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::{fmt, marker::PhantomData};

mod code;
pub use code::*;
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
    Text(Text),
    DecoratedText(DecoratedText),
    Keyword(Keyword),
    Link(Link),
    Tags(Tags),
    Code(CodeInline),
    Math(MathInline),
}

/// Represents a wrapper around a `InlineElement` where we already know the
/// type it will be and can therefore convert to either the `InlineElement`
/// or the inner type
#[derive(Clone, Debug, Display, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[display(fmt = "{}", inner)]
pub struct TypedInlineElement<T> {
    inner: InlineElement,
    phantom: PhantomData<T>,
}

impl<T> TypedInlineElement<T> {
    pub fn into_inner(self) -> InlineElement {
        self.inner
    }

    pub fn as_inner(&self) -> &InlineElement {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut InlineElement {
        &mut self.inner
    }
}

macro_rules! typed_inline_element_impl {
    ($type:ty, $variant:ident, $name:ident) => {
        paste! {
            impl TypedInlineElement<$type> {
                pub fn [<from_ $name>](x: $type) -> Self {
                    Self {
                        inner: InlineElement::from(x),
                        phantom: PhantomData,
                    }
                }

                pub fn into_typed(self) -> $type {
                    match self.inner {
                        InlineElement::$variant(x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn as_typed(&self) -> &$type {
                    match self.inner {
                        InlineElement::$variant(ref x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn as_mut_typed(&mut self) -> &mut $type {
                    match self.inner {
                        InlineElement::$variant(ref mut x) => x,
                        _ => unreachable!(),
                    }
                }
            }
        }
    };
}

typed_inline_element_impl!(Text, Text, text);
typed_inline_element_impl!(DecoratedText, DecoratedText, decorated_text);
typed_inline_element_impl!(Keyword, Keyword, keyword);
typed_inline_element_impl!(Link, Link, link);
typed_inline_element_impl!(Tags, Tags, tags);
typed_inline_element_impl!(CodeInline, Code, code_inline);
typed_inline_element_impl!(MathInline, Math, math_inline);

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
        Self::from(element.map(|x| Text::new(x.to_string())))
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

container_mapping!(LE<CodeInline>);
container_mapping!(LE<MathInline>);
container_mapping!(LE<Text>);
container_mapping!(LE<DecoratedText>);
container_mapping!(LE<Keyword>);
container_mapping!(LE<Link>);
container_mapping!(LE<Tags>);
