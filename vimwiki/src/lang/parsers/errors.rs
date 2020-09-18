use super::Span;
use nom::error::{ErrorKind, ParseError};
use std::fmt;

/// Represents an encapsulated error that is encountered
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LangParserError {
    ctx: String,
    sample: String,
    offset: usize,
    line: u32,
    column: usize,
    next: Option<Box<Self>>,
}

impl fmt::Display for LangParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: Line {}, Column {}",
            self.ctx, self.line, self.column
        )?;
        writeln!(f, "Input: {}", self.sample)?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        }

        Ok(())
    }
}

impl std::error::Error for LangParserError {}

impl LangParserError {
    pub fn unsupported() -> Self {
        Self {
            ctx: "Unsupported".to_string(),
            sample: "".to_string(),
            offset: 0,
            line: 0,
            column: 0,
            next: None,
        }
    }

    pub fn from_ctx(input: &Span, ctx: &'static str) -> Self {
        let line = input.global_line();
        let column = input.global_utf8_column();
        Self {
            ctx: ctx.to_string(),
            sample: input
                .fragment_str()
                .get(..16)
                .map(|x| x.to_string())
                .unwrap_or_default(),
            offset: input.global_offset(),
            line,
            column,
            next: None,
        }
    }
}

impl ParseError<Span> for LangParserError {
    fn from_error_kind(input: Span, kind: ErrorKind) -> Self {
        let line = input.global_line();
        let column = input.global_utf8_column();
        Self {
            ctx: kind.description().to_string(),
            sample: input
                .fragment_str()
                .get(..16)
                .map(|x| x.to_string())
                .unwrap_or_default(),
            offset: input.global_offset(),
            line,
            column,
            next: None,
        }
    }

    fn append(input: Span, kind: ErrorKind, other: Self) -> Self {
        let mut e = Self::from_error_kind(input, kind);
        e.next = Some(Box::new(other));
        e
    }

    fn from_char(input: Span, c: char) -> Self {
        let line = input.global_line();
        let column = input.global_utf8_column();
        Self {
            ctx: format!("Char {}", c),
            sample: input
                .fragment_str()
                .get(..16)
                .map(|x| x.to_string())
                .unwrap_or_default(),
            offset: input.global_offset(),
            line,
            column,
            next: None,
        }
    }

    fn or(self, other: Self) -> Self {
        // Pick error that has progressed further
        if self.offset > other.offset {
            self
        } else {
            other
        }
    }

    fn add_context(input: Span, ctx: &'static str, other: Self) -> Self {
        let line = input.global_line();
        let column = input.global_utf8_column();
        Self {
            ctx: ctx.to_string(),
            sample: input
                .fragment_str()
                .get(..16)
                .map(|x| x.to_string())
                .unwrap_or_default(),
            offset: input.global_offset(),
            line,
            column,
            next: Some(Box::new(other)),
        }
    }
}
