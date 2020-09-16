use super::Span;
use nom::{HexDisplay, IResult};
use std::fmt::Debug;

/// Port of nom's `dbg_dmp` to support a `Span`
#[allow(dead_code)]
pub fn dbg_dmp<'a, F, O, E: Debug>(
    f: F,
    context: &'static str,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O, E>
where
    F: Fn(Span<'a>) -> IResult<Span<'a>, O, E>,
{
    move |s: Span<'a>| match f(s) {
        Err(e) => {
            println!(
                "{}: Error({:?}) at:\n{}",
                context,
                e,
                (**s.fragment()).to_hex(8)
            );
            Err(e)
        }
        a => a,
    }
}
