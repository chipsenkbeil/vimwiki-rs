use super::{InlineElement, Region};
use vimwiki::{elements, LC};

/// Represents a single document list
#[derive(async_graphql::SimpleObject, Debug)]
pub struct List {
    /// The segment of the document this list covers
    region: Region,

    /// The items contained in the list
    items: Vec<ListItem>,
}

impl From<LC<elements::List>> for List {
    fn from(mut lc: LC<elements::List>) -> Self {
        Self {
            region: Region::from(lc.region),
            items: lc.element.items.drain(..).map(ListItem::from).collect(),
        }
    }
}

/// Represents a single item within a list in a document
#[derive(async_graphql::SimpleObject, Debug)]
pub struct ListItem {
    /// The segment of the document this list item covers
    region: Region,

    /// The type of the list item
    item_type: ListItemType,

    /// The suffix to a list item
    suffix: ListItemSuffix,

    /// The position of this list item among all items in a list
    position: i32,

    /// The contents contained within the list item
    contents: Vec<ListItemContent>,

    /// Additional attributes associated with the list item
    attributes: Vec<ListItemAttribute>,
}

impl From<LC<elements::EnhancedListItem>> for ListItem {
    fn from(lc: LC<elements::EnhancedListItem>) -> Self {
        let region = Region::from(lc.region);
        let mut element = lc.element;

        Self {
            region,
            item_type: ListItemType::from(element.item.item_type),
            suffix: ListItemSuffix::from(element.item.suffix),
            position: element.item.pos as i32,
            contents: element
                .item
                .contents
                .contents
                .drain(..)
                .map(ListItemContent::from)
                .collect(),
            attributes: element
                .attributes
                .drain()
                .map(ListItemAttribute::from)
                .collect(),
        }
    }
}

/// Represents the type of prefix used with a list item
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListItemType {
    Number,
    Pound,
    LowercaseAlphabet,
    UppercaseAlphabet,
    LowercaseRoman,
    UppercaseRoman,
    Hyphen,
    Asterisk,
    Other,
}

impl From<elements::ListItemType> for ListItemType {
    fn from(t: elements::ListItemType) -> Self {
        match t {
            elements::ListItemType::Ordered(x) => match x {
                elements::OrderedListItemType::LowercaseAlphabet => {
                    Self::LowercaseAlphabet
                }
                elements::OrderedListItemType::LowercaseRoman => {
                    Self::LowercaseRoman
                }
                elements::OrderedListItemType::Number => Self::Number,
                elements::OrderedListItemType::Pound => Self::Pound,
                elements::OrderedListItemType::UppercaseAlphabet => {
                    Self::UppercaseAlphabet
                }
                elements::OrderedListItemType::UppercaseRoman => {
                    Self::UppercaseRoman
                }
            },
            elements::ListItemType::Unordered(x) => match x {
                elements::UnorderedListItemType::Asterisk => Self::Asterisk,
                elements::UnorderedListItemType::Hyphen => Self::Hyphen,
                elements::UnorderedListItemType::Other(_) => Self::Other,
            },
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListItemSuffix {
    None,
    Period,
    Paren,
}

impl From<elements::ListItemSuffix> for ListItemSuffix {
    fn from(s: elements::ListItemSuffix) -> Self {
        match s {
            elements::ListItemSuffix::None => Self::None,
            elements::ListItemSuffix::Paren => Self::Paren,
            elements::ListItemSuffix::Period => Self::Period,
        }
    }
}

#[derive(async_graphql::Union, Debug)]
pub enum ListItemContent {
    InlineContent(InlineContent),
    List(List),
}

impl From<LC<elements::ListItemContent>> for ListItemContent {
    fn from(lc: LC<elements::ListItemContent>) -> Self {
        match lc.element {
            elements::ListItemContent::InlineContent(mut x) => {
                Self::InlineContent(InlineContent {
                    elements: x
                        .elements
                        .drain(..)
                        .map(InlineElement::from)
                        .collect(),
                })
            }
            elements::ListItemContent::List(x) => {
                Self::List(List::from(LC::new(x, lc.region)))
            }
        }
    }
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct InlineContent {
    elements: Vec<InlineElement>,
}

#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListItemAttribute {
    /// Flags list item as a TODO item that has not been completed
    TodoIncomplete,

    /// Flags list item as a TODO item that is partially complete (1-33%)
    TodoPartiallyComplete1,

    /// Flags list item as a TODO item that is partially complete (34-66%)
    TodoPartiallyComplete2,

    /// Flags list item as a TODO item that is partially complete (67-99%)
    TodoPartiallyComplete3,

    /// Flags list item as a TODO item that is complete
    TodoComplete,

    /// Flags list item as a TODO item that has been rejected
    TodoRejected,
}

impl From<elements::EnhancedListItemAttribute> for ListItemAttribute {
    fn from(a: elements::EnhancedListItemAttribute) -> Self {
        match a {
            elements::EnhancedListItemAttribute::TodoIncomplete => {
                Self::TodoIncomplete
            }
            elements::EnhancedListItemAttribute::TodoPartiallyComplete1 => {
                Self::TodoPartiallyComplete1
            }
            elements::EnhancedListItemAttribute::TodoPartiallyComplete2 => {
                Self::TodoPartiallyComplete2
            }
            elements::EnhancedListItemAttribute::TodoPartiallyComplete3 => {
                Self::TodoPartiallyComplete3
            }
            elements::EnhancedListItemAttribute::TodoComplete => {
                Self::TodoComplete
            }
            elements::EnhancedListItemAttribute::TodoRejected => {
                Self::TodoRejected
            }
        }
    }
}
