use super::Located;
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
pub enum InlineElement<'a> {
    Text(Text<'a>),
    DecoratedText(DecoratedText<'a>),
    Keyword(Keyword),
    Link(Link<'a>),
    Tags(Tags<'a>),
    Code(CodeInline<'a>),
    Math(MathInline<'a>),
}

/// Represents a wrapper around a `InlineElement` where we already know the
/// type it will be and can therefore convert to either the `InlineElement`
/// or the inner type
#[derive(Clone, Debug, Display, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[display(fmt = "{}", inner)]
pub struct TypedInlineElement<'a, T> {
    inner: InlineElement<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> TypedInlineElement<'a, T> {
    pub fn into_inner(self) -> InlineElement<'a> {
        self.inner
    }

    pub fn as_inner(&self) -> &InlineElement {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut InlineElement<'a> {
        &mut self.inner
    }
}

macro_rules! typed_inline_element_impl {
    ($type:ty, $variant:ident, $name:ident) => {
        paste! {
            impl<'a> TypedInlineElement<'a, $type> {
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

typed_inline_element_impl!(Text<'a>, Text, text);
typed_inline_element_impl!(DecoratedText<'a>, DecoratedText, decorated_text);
typed_inline_element_impl!(Keyword, Keyword, keyword);
typed_inline_element_impl!(Link<'a>, Link, link);
typed_inline_element_impl!(Tags<'a>, Tags, tags);
typed_inline_element_impl!(CodeInline<'a>, Code, code_inline);
typed_inline_element_impl!(MathInline<'a>, Math, math_inline);

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
pub struct InlineElementContainer<'a> {
    pub elements: Vec<Located<InlineElement<'a>>>,
}

impl<'a> fmt::Display for InlineElementContainer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for le in self.elements.iter() {
            write!(f, "{}", le.element.to_string())?;
        }
        Ok(())
    }
}

impl<'a> From<Vec<InlineElementContainer<'a>>> for InlineElementContainer<'a> {
    fn from(mut containers: Vec<Self>) -> Self {
        Self::new(containers.drain(..).flat_map(|c| c.elements).collect())
    }
}

impl<'a> From<Located<InlineElement<'a>>> for InlineElementContainer<'a> {
    fn from(element: Located<InlineElement<'a>>) -> Self {
        Self::new(vec![element])
    }
}

impl<'a> From<Located<&'a str>> for InlineElementContainer<'a> {
    fn from(element: Located<&'a str>) -> Self {
        Self::from(element.map(|x| Text::from(x)))
    }
}

macro_rules! container_mapping {
    ($type:ty) => {
        impl<'a> From<$type> for InlineElementContainer<'a> {
            fn from(element: $type) -> Self {
                Self::from(element.map(InlineElement::from))
            }
        }
    };
}

container_mapping!(Located<CodeInline<'a>>);
container_mapping!(Located<MathInline<'a>>);
container_mapping!(Located<Text<'a>>);
container_mapping!(Located<DecoratedText<'a>>);
container_mapping!(Located<Keyword>);
container_mapping!(Located<Link<'a>>);
container_mapping!(Located<Tags<'a>>);
