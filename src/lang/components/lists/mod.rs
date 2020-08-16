use super::InlineComponent;
use derive_more::From;
use serde::{Deserialize, Serialize};

mod definition;
mod ordered;
mod unordered;

// TODO: How to configure this? have ListItem struct that can have a prefix
//       for ordered and unordered? Provide special increment/decrement
//       support for ordered? How do tasks/todos fit in? Special ListTodoItem?
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum List {
    // TODO: multiline lists can be broken up with sublists, which is what
    //       makes this harder; each list will need to have a starting line
    //       is guaranteed inline text and then a tail that can have zero or
    //       more lines that are either a) continued inline text or b) a sublist
    //
    //       The list structs should have a method that can collect all
    //       inline text without sublists as lines
    Unordered(),
    Ordered(),
    Definition(),
}
