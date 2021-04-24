use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_tags, Tags<'a>, 'a);
fn tokenize_tags(ctx: &TokenizeContext, tags: &Tags) -> TokenStream {
    let root = element_path();
    let inner = tags.0.iter().map(|x| tokenize_tag(ctx, x));
    quote! {
        #root::Tags(::std::vec![#(#inner),*])
    }
}

impl_tokenize!(tokenize_tag, Tag<'a>, 'a);
fn tokenize_tag(ctx: &TokenizeContext, tag: &Tag) -> TokenStream {
    let root = element_path();
    let inner = do_tokenize!(ctx, &tag.0);
    quote! {
        #root::Tag(#inner)
    }
}
