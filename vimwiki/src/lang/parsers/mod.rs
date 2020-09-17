// Import to make more easily accessible to submodules
use super::{
    components,
    utils::{Region, Span, SpanFactory, LC},
};

mod errors;
pub use errors::LangParserError;

mod utils;

pub mod vimwiki;
