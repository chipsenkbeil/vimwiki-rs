use super::InlineComponentContainer;
use derive_more::From;
use serde::{Deserialize, Serialize};

mod definition;
pub use definition::{Definition, DefinitionList, Term};

mod regular;
pub use regular::{
    EnhancedListItem, EnhancedListItemAttribute, List as RegularList, ListItem,
    ListItemContent, ListItemContents, OrderedListItem, OrderedListItemSuffix,
    OrderedListItemType, UnorderedListItem, UnorderedListItemType,
};

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum List {
    Regular(RegularList),
    Definition(DefinitionList),
}
