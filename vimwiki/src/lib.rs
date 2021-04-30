mod lang;
mod utils;

// Export all elements at top level
pub use lang::elements::*;

// Export all outputs at top level
pub use lang::outputs::*;

// Export our parser error, which is used for language parsing
pub use lang::parsers::Error as ParseError;

// Export our primary language structure and trait
pub use lang::{FromLanguage, Language};

// Export our trait to do stronger comparsisons that include the region of elements
pub use utils::StrictEq;

// Re-export the vendor libraries so we're able to reconstruct their
// structs from macros
pub mod vendor {
    pub use chrono;
    pub use uriparse;
}

#[cfg(feature = "timekeeper")]
pub mod timekeeper;
