use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::Divider;

impl_tokenize!(tokenize_divider, Divider);
fn tokenize_divider(_ctx: &TokenizeContext, _divider: &Divider) -> TokenStream {
    let root = root_crate();
    quote! {
        #root::Divider
    }
}
