use super::{InlineComponent, Region};
use vimwiki::{components, LC};

/// Represents a single document list
#[derive(async_graphql::SimpleObject)]
pub struct List {
    /// The segment of the document this list covers
    region: Region,

    /// The items contained in the list
    items: Vec<ListItem>,
}

impl From<LC<components::List>> for List {
    fn from(mut lc: LC<components::List>) -> Self {
        Self {
            region: Region::from(lc.region),
            items: lc.component.items.drain(..).map(ListItem::from).collect(),
        }
    }
}

/// Represents a single item within a list in a document
#[derive(async_graphql::SimpleObject)]
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

impl From<LC<components::EnhancedListItem>> for ListItem {
    fn from(lc: LC<components::EnhancedListItem>) -> Self {
        let region = Region::from(lc.region);
        let mut component = lc.component;

        Self {
            region,
            item_type: ListItemType::from(component.item.item_type),
            suffix: ListItemSuffix::from(component.item.suffix),
            position: component.item.pos as i32,
            contents: component
                .item
                .contents
                .contents
                .drain(..)
                .map(ListItemContent::from)
                .collect(),
            attributes: component
                .attributes
                .drain()
                .map(ListItemAttribute::from)
                .collect(),
        }
    }
}

/// Represents the type of prefix used with a list item
#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
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

impl From<components::ListItemType> for ListItemType {
    fn from(t: components::ListItemType) -> Self {
        match t {
            components::ListItemType::Ordered(x) => match x {
                components::OrderedListItemType::LowercaseAlphabet => {
                    Self::LowercaseAlphabet
                }
                components::OrderedListItemType::LowercaseRoman => {
                    Self::LowercaseRoman
                }
                components::OrderedListItemType::Number => Self::Number,
                components::OrderedListItemType::Pound => Self::Pound,
                components::OrderedListItemType::UppercaseAlphabet => {
                    Self::UppercaseAlphabet
                }
                components::OrderedListItemType::UppercaseRoman => {
                    Self::UppercaseRoman
                }
            },
            components::ListItemType::Unordered(x) => match x {
                components::UnorderedListItemType::Asterisk => Self::Asterisk,
                components::UnorderedListItemType::Hyphen => Self::Hyphen,
                components::UnorderedListItemType::Other(_) => Self::Other,
            },
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
pub enum ListItemSuffix {
    None,
    Period,
    Paren,
}

impl From<components::ListItemSuffix> for ListItemSuffix {
    fn from(s: components::ListItemSuffix) -> Self {
        match s {
            components::ListItemSuffix::None => Self::None,
            components::ListItemSuffix::Paren => Self::Paren,
            components::ListItemSuffix::Period => Self::Period,
        }
    }
}

#[derive(async_graphql::Union)]
pub enum ListItemContent {
    InlineContent(InlineContent),
    List(List),
}

impl From<LC<components::ListItemContent>> for ListItemContent {
    fn from(lc: LC<components::ListItemContent>) -> Self {
        match lc.component {
            components::ListItemContent::InlineContent(mut x) => {
                Self::InlineContent(InlineContent {
                    components: x
                        .components
                        .drain(..)
                        .map(InlineComponent::from)
                        .collect(),
                })
            }
            components::ListItemContent::List(x) => {
                Self::List(List::from(LC::new(x, lc.region)))
            }
        }
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct InlineContent {
    components: Vec<InlineComponent>,
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq)]
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

impl From<components::EnhancedListItemAttribute> for ListItemAttribute {
    fn from(a: components::EnhancedListItemAttribute) -> Self {
        match a {
            components::EnhancedListItemAttribute::TodoIncomplete => {
                Self::TodoIncomplete
            }
            components::EnhancedListItemAttribute::TodoPartiallyComplete1 => {
                Self::TodoPartiallyComplete1
            }
            components::EnhancedListItemAttribute::TodoPartiallyComplete2 => {
                Self::TodoPartiallyComplete2
            }
            components::EnhancedListItemAttribute::TodoPartiallyComplete3 => {
                Self::TodoPartiallyComplete3
            }
            components::EnhancedListItemAttribute::TodoComplete => {
                Self::TodoComplete
            }
            components::EnhancedListItemAttribute::TodoRejected => {
                Self::TodoRejected
            }
        }
    }
}
