use super::InlineComponentContainer;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod item;
pub use item::{
    ListItem, ListItemContent, ListItemContents, OrderedListItem,
    OrderedListItemSuffix, OrderedListItemType, UnorderedListItem,
    UnorderedListItemType,
};

#[derive(
    Constructor, Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct List {
    items: Vec<ListItem>,
}
