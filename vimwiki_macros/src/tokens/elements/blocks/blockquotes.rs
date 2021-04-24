use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_blockquote, Blockquote<'a>, 'a);
fn tokenize_blockquote(
    ctx: &TokenizeContext,
    blockquote: &Blockquote,
) -> TokenStream {
    let root = element_path();
    let lines = blockquote.lines().iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::Blockquote {
            lines: ::std::vec![#(#lines),*],
        }
    }
}
