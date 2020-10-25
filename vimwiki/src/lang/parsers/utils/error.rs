/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur
#[cfg(not(feature = "timekeeper"))]
pub use nom::error::context;

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur. This implementation also logs to a
/// timekeeper table, which can be printed out to evaluate the time spent
/// within each parser wrapped in a context.
#[cfg(feature = "timekeeper")]
pub fn context<'a, T>(
    ctx: &'static str,
    f: impl Fn(Span<'a>) -> IResult<'a, T>,
) -> impl Fn(Span<'a>) -> IResult<'a, T> {
    crate::timekeeper::parsers::context(ctx, f)
}
