use super::{Element, InlineElement, InlineElementContainer, Located};
use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};

mod item;
pub use item::*;

/// Represents a regular list comprised of individual items
#[derive(
    Constructor, Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct List<'a> {
    pub items: Vec<Located<ListItem<'a>>>,
}

impl List<'_> {
    pub fn to_borrowed(&self) -> List {
        List {
            items: self
                .items
                .iter()
                .map(|x| x.as_ref().map(ListItem::to_borrowed))
                .collect(),
        }
    }

    pub fn into_owned(self) -> List<'static> {
        List {
            items: self
                .items
                .into_iter()
                .map(|x| x.map(ListItem::into_owned))
                .collect(),
        }
    }
}

impl<'a> List<'a> {
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
                item.item_type = head.item_type.clone();
            }
        }

        self
    }

    pub fn to_children(&'a self) -> Vec<Located<Element<'a>>> {
        self.items
            .iter()
            .flat_map(|x| x.as_inner().to_children())
            .collect()
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

/// Represents a collection of list item content
#[derive(
    Constructor,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub struct ListItemContents<'a> {
    pub contents: Vec<Located<ListItemContent<'a>>>,
}

impl ListItemContents<'_> {
    pub fn to_borrowed(&self) -> ListItemContents {
        ListItemContents {
            contents: self
                .contents
                .iter()
                .map(|x| x.as_ref().map(ListItemContent::to_borrowed))
                .collect(),
        }
    }

    pub fn into_owned(self) -> ListItemContents<'static> {
        ListItemContents {
            contents: self
                .contents
                .into_iter()
                .map(|x| x.map(ListItemContent::into_owned))
                .collect(),
        }
    }
}

impl<'a> ListItemContents<'a> {
    pub fn inline_content_iter(
        &self,
    ) -> impl Iterator<Item = &InlineElement> + '_ {
        self.contents
            .iter()
            .filter_map(|c| match c.as_inner() {
                ListItemContent::InlineContent(x) => {
                    Some(x.elements.iter().map(|y| y.as_inner()))
                }
                _ => None,
            })
            .flatten()
    }

    pub fn inline_content_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut InlineElement<'a>> + '_ {
        self.contents
            .iter_mut()
            .filter_map(|c| match c.as_mut_inner() {
                ListItemContent::InlineContent(x) => {
                    Some(x.elements.iter_mut().map(|y| y.as_mut_inner()))
                }
                _ => None,
            })
            .flatten()
    }

    pub fn sublist_iter(&self) -> impl Iterator<Item = &List> + '_ {
        self.contents.iter().flat_map(|c| match c.as_inner() {
            ListItemContent::List(x) => Some(x),
            _ => None,
        })
    }

    pub fn sublist_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut List<'a>> + '_ {
        self.contents
            .iter_mut()
            .flat_map(|c| match c.as_mut_inner() {
                ListItemContent::List(x) => Some(x),
                _ => None,
            })
    }

    pub fn to_children(&'a self) -> Vec<Located<Element<'a>>> {
        self.contents
            .iter()
            .flat_map(|x| match x.as_inner() {
                ListItemContent::InlineContent(content) => content
                    .to_children()
                    .into_iter()
                    .map(|x| x.map(Element::from))
                    .collect(),
                ListItemContent::List(list) => vec![Located::new(
                    Element::from(list.to_borrowed()),
                    x.region(),
                )],
            })
            .collect()
    }
}
