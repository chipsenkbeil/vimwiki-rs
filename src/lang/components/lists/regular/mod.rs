use super::InlineComponentContainer;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod item;
pub use item::{
    EnhancedListItem, EnhancedListItemAttribute, ListItem, OrderedListItem,
    OrderedListItemSuffix, OrderedListItemType, UnorderedListItem,
    UnorderedListItemType,
};

/// Represents a regular list comprised of individual items
#[derive(
    Constructor, Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct List {
    pub items: Vec<EnhancedListItem>,
}

/// Represents some content associated with a list item, either being
/// an inline component or a new sublist
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum ListItemContent {
    InlineContent(InlineComponentContainer),
    List(List),
}

/// Represents a collection of list item content
pub type ListItemContents = Vec<ListItemContent>;
