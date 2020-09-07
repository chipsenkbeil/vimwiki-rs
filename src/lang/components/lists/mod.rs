use super::{InlineComponent, InlineComponentContainer, LC};
use derive_more::From;
use serde::{Deserialize, Serialize};

mod definition;
pub use definition::{Definition, DefinitionList, Term};

mod regular;
pub use regular::*;

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum List {
    Regular(RegularList),
    Definition(DefinitionList),
}
