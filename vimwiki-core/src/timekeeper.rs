use crate::lang::parsers::Span;
use lazy_static::lazy_static;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

lazy_static! {
    static ref TIMEKEEPER_ENABLED: AtomicBool = AtomicBool::new(false);
    static ref TIMEKEEPER: Mutex<std::collections::HashMap<&'static str, (usize, u128)>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

pub fn is_enabled() -> bool {
    TIMEKEEPER_ENABLED.load(Ordering::Relaxed)
}

pub fn enable() {
    TIMEKEEPER_ENABLED.store(true, Ordering::Relaxed);
}

pub fn disable() {
    TIMEKEEPER_ENABLED.store(false, Ordering::Relaxed);
}

pub fn toggle_enabled() {
    if is_enabled() {
        disable();
    } else {
        enable();
    }
}

/// Prints a report based on the timekeeper's memory
pub fn print_report(clear_after_print: bool) {
    let mut results: Vec<(&'static str, (usize, u128))> = TIMEKEEPER
        .lock()
        .unwrap()
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();

    // Sort with most expensive average item first
    results.sort_unstable_by_key(|k| (k.1 .1 as f64 / k.1 .0 as f64) as u128);
    results.reverse();

    fn time_to_str(x: u128) -> String {
        if x >= 10_u128.pow(9) {
            format!("{}s", (x as f64) / 10_f64.powi(9))
        } else if x >= 10_u128.pow(6) {
            format!("{}ms", (x as f64) / 10_f64.powi(6))
        } else if x >= 10_u128.pow(3) {
            format!("{}μs", (x as f64) / 10_f64.powi(3))
        } else {
            format!("{}ns", x)
        }
    }

    println!("====== TIMEKEEPER REPORT ======");
    println!();
    for (ctx, (cnt, nanos)) in results.drain(..) {
        println!(
            "- {}: ({} calls, total {}, average {})",
            ctx,
            cnt,
            time_to_str(nanos),
            time_to_str((nanos as f64 / cnt as f64) as u128),
        );
    }
    println!();
    println!("===============================");

    if clear_after_print {
        clear();
    }
}

/// Clears the timekeeper's memory
pub fn clear() {
    TIMEKEEPER.lock().unwrap().clear();
}

/// Wraps a parser in a contextual label, which makes it easier to identify
/// where parsing failures occur. This implementation also logs to a
/// timekeeper table, which can be printed out to evaluate the time spent
/// within each parser wrapped in a context.
pub mod parsers {
    use super::*;
    use nom::error::ContextError;

    type Error<'a> = crate::lang::parsers::Error<'a>;
    type IResult<'a, O> = Result<(Span<'a>, O), nom::Err<Error<'a>>>;

    pub fn context<'a, T>(
        ctx: &'static str,
        mut f: impl FnMut(Span<'a>) -> IResult<T>,
    ) -> impl FnMut(Span<'a>) -> IResult<T> {
        move |input: Span| {
            let start = std::time::Instant::now();

            // NOTE: Following is the code found in nom's context parser, but due
            //       to issues with wrapping a function like above in a parser,
            //       we have to explicitly call the f parser on its own
            let result = match f(input) {
                Ok(o) => Ok(o),
                Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
                Err(nom::Err::Error(e)) => {
                    Err(nom::Err::Error(Error::add_context(input, ctx, e)))
                }
                Err(nom::Err::Failure(e)) => {
                    Err(nom::Err::Failure(Error::add_context(input, ctx, e)))
                }
            };

            if is_enabled() {
                let x = start.elapsed().as_nanos();
                TIMEKEEPER
                    .lock()
                    .unwrap()
                    .entry(ctx)
                    .and_modify(move |e| {
                        *e = (e.0 + 1, e.1 + x);
                    })
                    .or_insert((1, x));
            }

            result
        }
    }
}
