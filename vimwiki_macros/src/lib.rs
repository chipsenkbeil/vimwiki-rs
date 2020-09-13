use proc_macro2::{
    token_stream::IntoIter as TokenIter, Span, TokenStream, TokenTree,
};
use vimwiki::{Parser, VimwikiParser};

mod error;
use error::{Error, Result};

mod tokens;
use tokens::Tokenize;

#[proc_macro]
pub fn vimwiki(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let output = expand(input);

    proc_macro::TokenStream::from(output)
}

fn expand(input: TokenStream) -> TokenStream {
    match try_expand(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn try_expand(input: TokenStream) -> Result<TokenStream> {
    let mut input = input.into_iter();

    let first = input.next().ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "unexpected end of macro invocation, expected format string",
        )
    })?;

    let raw_source = input_to_string(first)?;

    // Options for reporting errors is nicely defined in this StackOverflow
    // answer: https://stackoverflow.com/a/54394014
    //
    // TLDR; Diagnostic API will be the better way in the future, but it is
    //       still a nightly-only feature. For now, we'll wrap a compile_error
    //       within a quote_spanned
    let located_page = VimwikiParser::parse_str(&raw_source)
        .map_err(|x| Error::new(Span::call_site(), &format!("{}", x)))?;

    require_empty_or_trailing_comma(&mut input)?;

    let mut stream = TokenStream::new();
    located_page.tokenize(&mut stream);
    Ok(stream)
}

fn input_to_string(token: TokenTree) -> Result<String> {
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

fn require_empty_or_trailing_comma(input: &mut TokenIter) -> Result<()> {
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
