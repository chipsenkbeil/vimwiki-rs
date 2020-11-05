use super::Span;
use nom::error::{ErrorKind, ParseError};
use std::{borrow::Cow, fmt};

/// Represents an encapsulated error that is encountered
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LangParserError<'a> {
    ctx: Cow<'a, str>,
    input: Span<'a>,
    next: Option<Box<Self>>,
}

impl<'a> From<nom::Err<LangParserError<'a>>> for LangParserError<'a> {
    fn from(nom_err: nom::Err<LangParserError<'a>>) -> Self {
        match nom_err {
            nom::Err::Error(x) | nom::Err::Failure(x) => x,
            nom::Err::Incomplete(_) => {
                Self::from_ctx(&Span::default(), "Incomplete")
            }
        }
    }
}

impl<'a> fmt::Display for LangParserError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display our context along with the starting line/column
        // NOTE: This is an expensive operation to calculate the line/column
        writeln!(
            f,
            "{}: Line {}, Column {}",
            self.ctx,
            self.input.line(),
            self.input.column()
        )?;

        // Produce the first line of our input, limiting to no more than
        // 100 characters to prevent really long lines
        writeln!(
            f,
            "{}",
            &self
                .input
                .as_unsafe_remaining_str()
                .lines()
                .next()
                .unwrap_or_default()
                .chars()
                .take(100)
                .collect::<String>()
        )?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        }

        Ok(())
    }
}

impl<'a> std::error::Error for LangParserError<'a> {}

impl<'a> LangParserError<'a> {
    pub fn unsupported() -> Self {
        Self {
            ctx: Cow::from("Unsupported"),
            input: Span::from(""),
            next: None,
        }
    }

    pub fn from_ctx(input: &Span<'a>, ctx: &'static str) -> Self {
        Self {
            ctx: Cow::from(ctx),
            input: *input,
            next: None,
        }
    }
}

impl<'a> ParseError<Span<'a>> for LangParserError<'a> {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        Self {
            ctx: Cow::from(kind.description().to_string()),
            input,
            next: None,
        }
    }

    fn append(input: Span<'a>, kind: ErrorKind, other: Self) -> Self {
        let mut e = Self::from_error_kind(input, kind);
        e.next = Some(Box::new(other));
        e
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self {
            ctx: Cow::from(format!("Char {}", c)),
            input,
            next: None,
        }
    }

    fn or(self, other: Self) -> Self {
        // Pick error that has progressed further
        if self.input.start_offset() > other.input.start_offset() {
            self
        } else {
            other
        }
    }

    fn add_context(input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        Self {
            ctx: Cow::from(ctx),
            input,
            next: Some(Box::new(other)),
        }
    }
}
