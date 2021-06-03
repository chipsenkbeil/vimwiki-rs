use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use vimwiki_core::Blockquote;

impl_tokenize!(tokenize_blockquote, Blockquote<'a>, 'a);
fn tokenize_blockquote(
    ctx: &TokenizeContext,
    blockquote: &Blockquote,
) -> TokenStream {
    let root = root_crate();
    let lines = blockquote
        .lines()
        .map(|x| do_tokenize!(ctx, Cow::Borrowed(x)));
    quote! {
        #root::Blockquote::new(::std::vec![#(#lines),*])
    }
}
