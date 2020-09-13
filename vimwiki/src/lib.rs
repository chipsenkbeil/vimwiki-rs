pub mod lang;

// Export the language components without dumping them into the top leve
pub use lang::components;

// Export our top-level parser structs
pub use lang::parsers::{vimwiki::VimwikiParser, LangParserError, Parser};

// Dump our utilities (LocatedComponent, Region, Point, ...) into top level
pub use lang::utils::*;

// Re-export the URI library so we're able to reconstruct it from macros
pub use uriparse as uri;
