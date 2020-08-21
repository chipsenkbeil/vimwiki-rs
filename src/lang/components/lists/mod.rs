use super::InlineComponentContainer;
use derive_more::From;
use serde::{Deserialize, Serialize};

mod definition;
pub use definition::{Definition, DefinitionList, Term};

mod regular;
pub use regular::{
    RegularList, RegularListItem, RegularListItemContent,
    RegularListItemContents, RegularListItemSuffix,
};

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum List {
    Regular(RegularList),
    Definition(DefinitionList),
}
