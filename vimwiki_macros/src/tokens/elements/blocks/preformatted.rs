use crate::tokens::{
    root_crate,
    utils::{tokenize_hashmap, tokenize_option, tokenize_string_type},
    Tokenize,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_preformatted_text, PreformattedText<'a>, 'a);
fn tokenize_preformatted_text(
    preformatted_text: &PreformattedText,
) -> TokenStream {
    let root = root_crate();
    let lang = tokenize_option(&preformatted_text.lang, tokenize_string);
    let metadata = tokenize_hashmap(
        &preformatted_text.metadata,
        tokenize_string_type(),
        tokenize_string_type(),
        |x| do_tokenize!(x),
        |x| do_tokenize!(x),
    );
    let lines = preformatted_text.lines.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::elements::PreformattedText {
            lang: #lang,
            metadata: #metadata,
            lines: vec![#(#lines),*],
        }
    }
}
