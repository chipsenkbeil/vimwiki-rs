use crate::tokens::{
    utils::{
        root_crate, tokenize_cow_str_type, tokenize_hashmap, tokenize_option,
    },
    Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::PreformattedText;

impl_tokenize!(tokenize_preformatted_text, PreformattedText<'a>, 'a);
fn tokenize_preformatted_text(
    ctx: &TokenizeContext,
    preformatted_text: &PreformattedText,
) -> TokenStream {
    let root = root_crate();
    let lang = tokenize_option(ctx, &preformatted_text.lang, |ctx, x| {
        do_tokenize!(ctx, x)
    });
    let metadata = tokenize_hashmap(
        &preformatted_text.metadata,
        tokenize_cow_str_type(),
        tokenize_cow_str_type(),
        |x| do_tokenize!(ctx, x),
        |x| do_tokenize!(ctx, x),
    );
    let lines = preformatted_text.lines.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::PreformattedText {
            lang: #lang,
            metadata: #metadata,
            lines: ::std::vec![#(#lines),*],
        }
    }
}
