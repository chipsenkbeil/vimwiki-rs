use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_tags, Tags<'a>, 'a);
fn tokenize_tags(tags: &Tags) -> TokenStream {
    let root = root_crate();
    let inner = tags.0.iter().map(tokenize_tag);
    quote! {
        #root::elements::Tags(vec![#(#inner),*])
    }
}

impl_tokenize!(tokenize_tag, Tag<'a>, 'a);
fn tokenize_tag(tag: &Tag) -> TokenStream {
    let root = root_crate();
    let inner = do_tokenize!(&tag.0);
    quote! {
        #root::elements::Tag(#inner)
    }
}
