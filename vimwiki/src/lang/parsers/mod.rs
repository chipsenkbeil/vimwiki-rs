mod errors;
mod span;
mod utils;

/// Vimwiki-specific parsers
pub mod vimwiki;

/// Export the span used for input
pub use span::Span;

/// Alias to the type of error to use with parsing using nom
pub use errors::LangParserError as Error;

/// Alias to an Result using our custom error and span
pub type IResult<'a, O> = Result<(Span<'a>, O), nom::Err<Error>>;

/// Represents some data captured with the input used to create it
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Captured<'a, T> {
    inner: T,
    input: Span<'a>,
}

impl<'a, T> Captured<'a, T> {
    pub fn new(inner: T, input: Span<'a>) -> Self {
        Self { inner, input }
    }

    /// Converts `Captured<'a, T>` to `Captured<'a, U>`
    ///
    /// NOTE: This should only be used to wrap the inner type in situations
    ///       where you have an enum that can contain the inner type;
    ///       converting to an arbitrary type that doesn't correspond
    ///       to the underlying inner input will cause problems
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Captured<'a, U> {
        Captured {
            inner: f(self.inner),
            input: self.input,
        }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Represents the input that was used to construct the data
    pub fn input(&self) -> Span<'a> {
        self.input
    }
}
