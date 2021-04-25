use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::Page;

pub mod blocks;
pub mod location;

// Top-level types
impl_tokenize!(tokenize_page, Page<'a>, 'a);
fn tokenize_page(ctx: &TokenizeContext, page: &Page) -> TokenStream {
    let root = root_crate();
    let elements = page.elements().iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::Page {
            elements: ::std::vec![#(#elements),*],
        }
    }
}
