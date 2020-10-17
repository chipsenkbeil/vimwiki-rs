use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_blockquote, Blockquote<'a>, 'a);
fn tokenize_blockquote(blockquote: &Blockquote) -> TokenStream {
    let root = root_crate();
    let lines = blockquote.lines.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::elements::Blockquote {
            lines: vec![#(#lines),*],
        }
    }
}
