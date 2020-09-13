use crate::error::{Error, Result};
use proc_macro2::{token_stream::IntoIter as TokenIter, TokenTree};

pub fn input_to_string(token: TokenTree) -> Result<String> {
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

    let begin = repr.find('"').unwrap() + 1;
    let end = repr.rfind('"').unwrap();
    Ok(repr[begin..end].to_string())
}

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
