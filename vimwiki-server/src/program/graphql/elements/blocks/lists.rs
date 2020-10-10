use super::{InlineElement, Region};
use vimwiki::{elements, LE};

/// Represents a single document list
#[derive(async_graphql::SimpleObject, Debug)]
pub struct List {
    /// The segment of the document this list covers
    region: Region,

    /// The items contained in the list
    items: Vec<ListItem>,
}

impl From<LE<elements::List>> for List {
    fn from(mut le: LE<elements::List>) -> Self {
        Self {
            region: Region::from(le.region),
            items: le.element.items.drain(..).map(ListItem::from).collect(),
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
    attributes: ListItemAttributes,
}

impl From<LE<elements::ListItem>> for ListItem {
    fn from(le: LE<elements::ListItem>) -> Self {
        let region = Region::from(le.region);
        let mut item = le.element;

        Self {
            region,
            item_type: ListItemType::from(item.item_type),
            suffix: ListItemSuffix::from(item.suffix),
            position: item.pos as i32,
            contents: item
                .contents
                .contents
                .drain(..)
                .map(ListItemContent::from)
                .collect(),
            attributes: ListItemAttributes::from(item.attributes),
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

impl From<LE<elements::ListItemContent>> for ListItemContent {
    fn from(le: LE<elements::ListItemContent>) -> Self {
        match le.element {
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
                Self::List(List::from(LE::new(x.into_typed(), le.region)))
            }
        }
    }
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct InlineContent {
    elements: Vec<InlineElement>,
}

#[derive(async_graphql::SimpleObject, Debug)]
pub struct ListItemAttributes {
    todo_status: Option<ListItemTodoStatus>,
}

impl From<elements::ListItemAttributes> for ListItemAttributes {
    fn from(x: elements::ListItemAttributes) -> Self {
        Self {
            todo_status: x.todo_status.map(ListItemTodoStatus::from),
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(remote = "vimwiki::elements::ListItemTodoStatus")]
pub enum ListItemTodoStatus {
    /// Flags list item as a TODO item that has not been completed
    Incomplete,

    /// Flags list item as a TODO item that is partially complete (1-33%)
    PartiallyComplete1,

    /// Flags list item as a TODO item that is partially complete (34-66%)
    PartiallyComplete2,

    /// Flags list item as a TODO item that is partially complete (67-99%)
    PartiallyComplete3,

    /// Flags list item as a TODO item that is complete
    Complete,

    /// Flags list item as a TODO item that has been rejected
    Rejected,
}
