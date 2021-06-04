use crate::{
    lang::elements::{
        AsChildrenMutSlice, AsChildrenSlice, Element, InlineBlockElement,
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{
    AsRef, Constructor, Deref, DerefMut, From, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

mod item;
pub use item::*;

/// Represents a regular list comprised of individual items
#[derive(
    Constructor,
    Clone,
    Debug,
    From,
    Eq,
    PartialEq,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
#[into_iterator(owned, ref, ref_mut)]
pub struct List<'a>(Vec<Located<ListItem<'a>>>);

impl List<'_> {
    pub fn to_borrowed(&self) -> List {
        self.into_iter()
            .map(|x| x.as_ref().map(ListItem::to_borrowed))
            .collect()
    }

    pub fn into_owned(self) -> List<'static> {
        self.into_iter()
            .map(|x| x.map(ListItem::into_owned))
            .collect()
    }
}

impl<'a> List<'a> {
    /// Returns iterator of references to list items
    pub fn iter(&self) -> impl Iterator<Item = &Located<ListItem<'a>>> {
        self.into_iter()
    }

    /// Returns iterator of mutable references to list items
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Located<ListItem<'a>>> {
        self.into_iter()
    }

    /// Returns total items contained in list
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if list has no items
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns whether or not the list represents an ordered list based on
    /// the first list item; if there are no items then this would return false
    pub fn is_ordered(&self) -> bool {
        self.iter()
            .next()
            .map_or(false, |item| item.ty().is_ordered())
    }

    /// Returns whether or not the list represents an unordered list based on
    /// the first list item; if there are no items then this would return false
    pub fn is_unordered(&self) -> bool {
        self.iter()
            .next()
            .map_or(false, |item| item.ty().is_unordered())
    }

    /// Normalizes the list by standardizing the item types based on the
    /// first list item.
    ///
    /// For example, if you have the following list:
    ///
    /// 1. Hyphen
    /// 2. Number
    /// 3. Asterisk
    ///
    /// You would get back out the following list:
    ///
    /// 1. Hyphen
    /// 2. Hyphen
    /// 3. Hyphen
    ///
    /// Note that this does NOT normalize sublists, which should be done
    /// manually.
    pub(crate) fn normalize(&mut self) -> &mut Self {
        // If we have items, we want to go through and normalize their types
        if let [head, tail @ ..] = &mut self.0[..] {
            // TODO: Need to support special case where not all item types are
            //       roman numeral but the first one is, as this can happen with
            //       alphabetic lists if for some reason starting with i and moving
            //       on to other letters like j and k
            for item in tail {
                *item.mut_ty() = head.ty().clone();
            }
        }

        self
    }
}

impl<'a> AsChildrenSlice for List<'a> {
    type Child = Located<ListItem<'a>>;

    fn as_children_slice(&self) -> &[Self::Child] {
        &self.0
    }
}

impl<'a> AsChildrenMutSlice for List<'a> {
    type Child = Located<ListItem<'a>>;

    fn as_children_mut_slice(&mut self) -> &mut [Self::Child] {
        &mut self.0
    }
}

impl<'a> IntoChildren for List<'a> {
    type Child = Located<InlineBlockElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.into_iter()
            .map(|x| x.map(InlineBlockElement::from))
            .collect()
    }
}

impl<'a> FromIterator<Located<ListItem<'a>>> for List<'a> {
    fn from_iter<I: IntoIterator<Item = Located<ListItem<'a>>>>(
        iter: I,
    ) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> StrictEq for List<'a> {
    /// Performs a strict_eq check against list items
    fn strict_eq(&self, other: &Self) -> bool {
        self.0.strict_eq(&other.0)
    }
}

/// Represents some content associated with a list item, either being
/// an inline element or a new sublist
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemContent<'a> {
    InlineContent(InlineElementContainer<'a>),
    List(List<'a>),
}

impl ListItemContent<'_> {
    pub fn to_borrowed(&self) -> ListItemContent {
        match self {
            Self::InlineContent(ref x) => {
                ListItemContent::from(x.to_borrowed())
            }
            Self::List(ref x) => ListItemContent::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> ListItemContent<'static> {
        match self {
            Self::InlineContent(x) => ListItemContent::from(x.into_owned()),
            Self::List(x) => ListItemContent::from(x.into_owned()),
        }
    }
}

impl<'a> StrictEq for ListItemContent<'a> {
    /// Performs a strict_eq check against eqivalent variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InlineContent(x), Self::InlineContent(y)) => x.strict_eq(y),
            (Self::List(x), Self::List(y)) => x.strict_eq(y),
            _ => false,
        }
    }
}

/// Represents a collection of list item content
#[derive(
    AsRef,
    Constructor,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
#[into_iterator(owned, ref, ref_mut)]
pub struct ListItemContents<'a>(Vec<Located<ListItemContent<'a>>>);

impl ListItemContents<'_> {
    pub fn to_borrowed(&self) -> ListItemContents {
        self.iter()
            .map(|x| x.as_ref().map(ListItemContent::to_borrowed))
            .collect()
    }

    pub fn into_owned(self) -> ListItemContents<'static> {
        self.into_iter()
            .map(|x| x.map(ListItemContent::into_owned))
            .collect()
    }
}

impl<'a> ListItemContents<'a> {
    pub fn inline_content_iter(
        &self,
    ) -> impl Iterator<Item = &InlineElement> + '_ {
        self.iter()
            .filter_map(|c| match c.as_inner() {
                ListItemContent::InlineContent(x) => {
                    Some(x.iter().map(|y| y.as_inner()))
                }
                _ => None,
            })
            .flatten()
    }

    pub fn inline_content_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut InlineElement<'a>> + '_ {
        self.iter_mut()
            .filter_map(|c| match c.as_mut_inner() {
                ListItemContent::InlineContent(x) => {
                    Some(x.iter_mut().map(|y| y.as_mut_inner()))
                }
                _ => None,
            })
            .flatten()
    }

    pub fn sublist_iter(&self) -> impl Iterator<Item = &List> + '_ {
        self.iter().flat_map(|c| match c.as_inner() {
            ListItemContent::List(x) => Some(x),
            _ => None,
        })
    }

    pub fn sublist_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut List<'a>> + '_ {
        self.iter_mut().flat_map(|c| match c.as_mut_inner() {
            ListItemContent::List(x) => Some(x),
            _ => None,
        })
    }
}

impl<'a> AsChildrenSlice for ListItemContents<'a> {
    type Child = Located<ListItemContent<'a>>;

    fn as_children_slice(&self) -> &[Self::Child] {
        &self.0
    }
}

impl<'a> AsChildrenMutSlice for ListItemContents<'a> {
    type Child = Located<ListItemContent<'a>>;

    fn as_children_mut_slice(&mut self) -> &mut [Self::Child] {
        &mut self.0
    }
}

impl<'a> IntoChildren for ListItemContents<'a> {
    type Child = Located<Element<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.into_iter()
            .flat_map(|x| {
                let region = x.region();
                match x.into_inner() {
                    ListItemContent::InlineContent(content) => content
                        .into_children()
                        .into_iter()
                        .map(|x| x.map(Element::from))
                        .collect(),
                    ListItemContent::List(list) => {
                        vec![Located::new(Element::from(list), region)]
                    }
                }
            })
            .collect()
    }
}

impl<'a> FromIterator<Located<ListItemContent<'a>>> for ListItemContents<'a> {
    fn from_iter<I: IntoIterator<Item = Located<ListItemContent<'a>>>>(
        iter: I,
    ) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> StrictEq for ListItemContents<'a> {
    /// Performs a strict_eq check against inner contents
    fn strict_eq(&self, other: &Self) -> bool {
        self.0.strict_eq(&other.0)
    }
}
