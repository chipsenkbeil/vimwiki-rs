use super::InlineComponentContainer;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod item;
pub use item::{
    RegularListItem, RegularListItemContent, RegularListItemContents,
    RegularListItemSuffix,
};

#[derive(
    Constructor, Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct RegularList {
    items: Vec<RegularListItem>,
}
