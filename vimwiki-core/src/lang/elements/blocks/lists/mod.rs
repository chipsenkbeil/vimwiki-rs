use crate::{
    lang::elements::{
        AsChildrenMutSlice, AsChildrenSlice, BlockElement, Element,
        InlineBlockElement, IntoChildren, Located,
    },
    ElementLike, StrictEq,
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
pub struct List<'a> {
    /// Represents items contained within the list
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub items: Vec<Located<ListItem<'a>>>,
}

impl ElementLike for List<'_> {}

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
        self.items.len()
    }

    /// Returns true if list has no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns whether or not the list represents an ordered list based on
    /// the first list item; if there are no items then this would return false
    pub fn is_ordered(&self) -> bool {
        self.iter()
            .next()
            .map_or(false, |item| item.ty.is_ordered())
    }

    /// Returns whether or not the list represents an unordered list based on
    /// the first list item; if there are no items then this would return false
    pub fn is_unordered(&self) -> bool {
        self.iter()
            .next()
            .map_or(false, |item| item.ty.is_unordered())
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
        if let [head, tail @ ..] = &mut self.items[..] {
            // TODO: Need to support special case where not all item types are
            //       roman numeral but the first one is, as this can happen with
            //       alphabetic lists if for some reason starting with i and moving
            //       on to other letters like j and k
            for item in tail {
                item.ty = head.ty.clone();
            }
        }

        self
    }
}

impl<'a> AsChildrenSlice for List<'a> {
    type Child = Located<ListItem<'a>>;

    fn as_children_slice(&self) -> &[Self::Child] {
        &self.items
    }
}

impl<'a> AsChildrenMutSlice for List<'a> {
    type Child = Located<ListItem<'a>>;

    fn as_children_mut_slice(&mut self) -> &mut [Self::Child] {
        &mut self.items
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
        self.items.strict_eq(&other.items)
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
pub struct ListItemContents<'a>(Vec<Located<BlockElement<'a>>>);

impl ElementLike for ListItemContents<'_> {}

impl ListItemContents<'_> {
    pub fn to_borrowed(&self) -> ListItemContents {
        self.iter()
            .map(|x| x.as_ref().map(BlockElement::to_borrowed))
            .collect()
    }

    pub fn into_owned(self) -> ListItemContents<'static> {
        self.into_iter()
            .map(|x| x.map(BlockElement::into_owned))
            .collect()
    }
}

impl<'a> ListItemContents<'a> {
    /// Represents an iterator over references to contents that are not sublists
    pub fn non_sublist_iter(&self) -> impl Iterator<Item = &BlockElement> + '_ {
        self.iter().filter_map(|c| {
            if matches!(c.as_inner(), BlockElement::List(_)) {
                None
            } else {
                Some(c.as_inner())
            }
        })
    }

    /// Represents an iterator over mutable references to contents that are not sublists
    pub fn non_sublist_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut BlockElement<'a>> + '_ {
        self.iter_mut().filter_map(|c| {
            if matches!(c.as_inner(), BlockElement::List(_)) {
                None
            } else {
                Some(c.as_mut_inner())
            }
        })
    }

    /// Represents an iterator over references to contents that are sublists
    pub fn sublist_iter(&self) -> impl Iterator<Item = &List> + '_ {
        self.iter().flat_map(|c| match c.as_inner() {
            BlockElement::List(x) => Some(x),
            _ => None,
        })
    }

    /// Represents an iterator over mut references to contents that are sublists
    pub fn sublist_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut List<'a>> + '_ {
        self.iter_mut().flat_map(|c| match c.as_mut_inner() {
            BlockElement::List(x) => Some(x),
            _ => None,
        })
    }
}

impl<'a> AsChildrenSlice for ListItemContents<'a> {
    type Child = Located<BlockElement<'a>>;

    fn as_children_slice(&self) -> &[Self::Child] {
        &self.0
    }
}

impl<'a> AsChildrenMutSlice for ListItemContents<'a> {
    type Child = Located<BlockElement<'a>>;

    fn as_children_mut_slice(&mut self) -> &mut [Self::Child] {
        &mut self.0
    }
}

impl<'a> IntoChildren for ListItemContents<'a> {
    type Child = Located<Element<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.into_iter().map(|x| x.map(Element::from)).collect()
    }
}

impl<'a> FromIterator<Located<BlockElement<'a>>> for ListItemContents<'a> {
    fn from_iter<I: IntoIterator<Item = Located<BlockElement<'a>>>>(
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
