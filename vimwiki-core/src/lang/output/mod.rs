#[cfg(feature = "html")]
mod html;

#[cfg(feature = "html")]
pub use html::*;

/// Represents the ability to convert some data into some other output form
pub trait Output {
    type Formatter;
    type Error;

    fn fmt(&self, f: &mut Self::Formatter) -> Result<(), Self::Error>;
}
