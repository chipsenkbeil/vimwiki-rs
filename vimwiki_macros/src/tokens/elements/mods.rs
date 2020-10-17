use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

mod blocks;
mod location;

// Top-level types
impl_tokenize!(tokenize_page, Page<'a>, 'a);
fn tokenize_page(page: &Page) -> TokenStream {
    let root = root_crate();
    let elements = page.elements.iter().map(|c| do_tokenize!(x));
    quote! {
        #root::elements::Page {
            elements: vec![#(#elements),*],
        }
    }
}
