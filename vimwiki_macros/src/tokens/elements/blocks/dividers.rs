use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_divider, Divider);
fn tokenize_divider(_divider: &Divider) -> TokenStream {
    let root = element_path();
    quote! {
        #root::Divider
    }
}
