use crate::error::{Error, Result};
use proc_macro2::{token_stream::IntoIter as TokenIter, Span, TokenTree};

/// Converts a token tree that is a string or byte string into a Rust string
/// instance. Removes any blank lines (whitespace only) before and after
/// lines with content.
///
/// If `raw_mode` is specified, will leave lines unaltered, otherwise will
/// find the minimum indentation level and remove that from all lines.
pub fn input_to_string(token: TokenTree, raw_mode: bool) -> Result<String> {
    let repr = token.to_string();
    let repr = repr.trim();
    let is_string = repr.starts_with('"') || repr.starts_with('r');
    let is_byte_string = repr.starts_with("b\"") || repr.starts_with("br");

    if !is_string && !is_byte_string {
        return Err(Error::new(
            token.span(),
            "argument must be a single string literal",
        ));
    }

    // Get the raw string as it appears
    let begin = repr.find('"').unwrap() + 1;
    let end = repr.rfind('"').unwrap();
    let s = repr[begin..end].to_string();

    // Determine which lines in the macro are blank and what each line's
    // indentation level is, which will be used if not in raw mode
    let mut line_data = s
        .lines()
        .map(move |l| {
            let is_blank = l.trim().is_empty();
            let indentation = l.len() - l.trim_start().len();
            (is_blank, indentation, l)
        })
        .collect::<Vec<(bool, usize, &str)>>();

    // Special handling to remove any blank lines at beginning and end of
    // the string; this includes lines that have whitespace but nothing else
    if !line_data.is_empty() {
        let mut start = 0;
        let mut end = line_data.len() - 1;

        // Keep moving backward while lines are blank and we haven't advanced
        // before the start
        while line_data[end].0 && end >= start {
            end -= 1;
        }

        // Keep moving forward while lines are blank and we haven't advanced
        // past the end
        while line_data[start].0 && start <= end {
            start += 1;
        }

        // Update line data to a subset
        if start <= end {
            line_data = line_data[start..=end].to_vec();
        } else {
            return Err(Error::new(
                Span::call_site(),
                &format!(
                    "Blank input provided! Need non-empty lines! {}/{}",
                    start, end
                ),
            ));
        }
    }

    // Process the lines back into a single string, either by doing nothing
    // to them or removing a set minimum indentation from all
    let lines =
        if raw_mode {
            line_data
                .iter()
                .map(|x| x.2)
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            let min_indentation = line_data
                .iter()
                .fold(usize::MAX, |acc, x| if x.1 < acc { x.1 } else { acc });
            line_data
                .iter()
                .map(|x| &x.2[min_indentation..])
                .collect::<Vec<&str>>()
                .join("\n")
        };

    Ok(lines)
}

/// Validates that our macro receives only a single argument, advancing
/// the input as a consequence
pub fn require_empty_or_trailing_comma(input: &mut TokenIter) -> Result<()> {
    let first = match input.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
            match input.next() {
                Some(second) => second,
                None => return Ok(()),
            }
        }
        Some(first) => first,
        None => return Ok(()),
    };
    let last = input.last();

    let begin_span = first.span();
    let end_span = last.as_ref().map_or(begin_span, TokenTree::span);
    let msg = format!(
        "unexpected {token} in macro invocation; vimwiki argument must be a single string literal",
        token = if last.is_some() { "tokens" } else { "token" }
    );
    Err(Error::new2(begin_span, end_span, &msg))
}
