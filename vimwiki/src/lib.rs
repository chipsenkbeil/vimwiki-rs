mod lang;
pub use lang::{
    elements::{self, Located, Region},
    parsers::Error as ParseError,
    FromLanguage, Language,
};

// Re-export the vendor libraries so we're able to reconstruct their
// structs from macros
pub mod vendor {
    pub use chrono;
    pub use uriparse;
}

mod tree;
pub use tree::{ElementTree, ElementTreeNode};

#[cfg(feature = "timekeeper")]
pub mod timekeeper;
