use crate::tokens::{
    utils::{
        element_path, tokenize_cow_str_type, tokenize_hashmap, tokenize_option,
    },
    Tokenize,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_preformatted_text, PreformattedText<'a>, 'a);
fn tokenize_preformatted_text(
    preformatted_text: &PreformattedText,
) -> TokenStream {
    let root = element_path();
    let lang = tokenize_option(&preformatted_text.lang, |x| do_tokenize!(x));
    let metadata = tokenize_hashmap(
        &preformatted_text.metadata,
        tokenize_cow_str_type(),
        tokenize_cow_str_type(),
        |x| do_tokenize!(x),
        |x| do_tokenize!(x),
    );
    let lines = preformatted_text.lines.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::PreformattedText {
            lang: #lang,
            metadata: #metadata,
            lines: vec![#(#lines),*],
        }
    }
}
