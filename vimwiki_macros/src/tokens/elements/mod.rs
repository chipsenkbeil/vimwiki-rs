use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

pub mod blocks;
pub mod location;

// Top-level types
impl_tokenize!(tokenize_page, Page<'a>, 'a);
fn tokenize_page(ctx: &TokenizeContext, page: &Page) -> TokenStream {
    let root = element_path();
    let elements = page.elements().iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::Page {
            elements: ::std::vec![#(#elements),*],
        }
    }
}
