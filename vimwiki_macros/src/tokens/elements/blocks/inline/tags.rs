use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_tags, Tags<'a>, 'a);
fn tokenize_tags(tags: &Tags) -> TokenStream {
    let root = element_path();
    let inner = tags.0.iter().map(tokenize_tag);
    quote! {
        #root::Tags(::std::vec![#(#inner),*])
    }
}

impl_tokenize!(tokenize_tag, Tag<'a>, 'a);
fn tokenize_tag(tag: &Tag) -> TokenStream {
    let root = element_path();
    let inner = do_tokenize!(&tag.0);
    quote! {
        #root::Tag(#inner)
    }
}
