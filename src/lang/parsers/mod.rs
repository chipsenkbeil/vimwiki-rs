// Import to make more easily accessible to submodules
use super::{
    components,
    utils::{Span, LC},
};
use derive_more::{Display, Error};

pub mod vimwiki;

/// Represents an encapsulated error that is encountered
#[derive(Clone, Debug, Eq, PartialEq, Display, Error)]
pub enum LangParserError<'a> {
    #[display(fmt = "Failure {:?} :: {}", error_kind, remaining)]
    Failure {
        remaining: &'a str,
        error_kind: nom::error::ErrorKind,
    },
    #[display(fmt = "Error {:?} :: {}", error_kind, remaining)]
    Error {
        remaining: &'a str,
        error_kind: nom::error::ErrorKind,
    },
    #[display(fmt = "Incomplete :: {}", size)]
    Incomplete { size: usize },
    #[display(fmt = "Incomplete :: ???")]
    IncompleteUnknown,
}
