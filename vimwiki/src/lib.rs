mod lang;
pub use lang::{
    elements::{self, Located, Region},
    parsers::Error as ParseError,
    FromLanguage, Language,
};

mod utils;
pub use utils::StrictEq;

// Re-export the vendor libraries so we're able to reconstruct their
// structs from macros
pub mod vendor {
    pub use chrono;
    pub use uriparse;
}

#[cfg(feature = "timekeeper")]
pub mod timekeeper;
