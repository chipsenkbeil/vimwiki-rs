#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::*;

mod vimwiki;
pub use vimwiki::*;

use std::{error::Error, fmt};

/// Represents the ability to convert some data into some other output form
pub trait Output<F: OutputFormatter> {
    /// Formats the value using the given formatter
    fn fmt(&self, f: &mut F) -> Result<(), F::Error>;
}

/// Represents a formatter used by some output struct
pub trait OutputFormatter: fmt::Write {
    /// Represents the types of errors that can be encountered when using
    /// this formatter
    type Error: Error;
}
