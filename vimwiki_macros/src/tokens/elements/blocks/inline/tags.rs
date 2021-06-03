use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use vimwiki_core::{Tag, Tags};

impl_tokenize!(tokenize_tags, Tags<'a>, 'a);
fn tokenize_tags(ctx: &TokenizeContext, tags: &Tags) -> TokenStream {
    let root = root_crate();
    let inner = tags.into_iter().map(|x| tokenize_tag(ctx, x));
    quote! {
        #root::Tags::new(::std::vec![#(#inner),*])
    }
}

impl_tokenize!(tokenize_tag, Tag<'a>, 'a);
fn tokenize_tag(ctx: &TokenizeContext, tag: &Tag) -> TokenStream {
    let root = root_crate();
    let inner = do_tokenize!(ctx, Cow::Borrowed(tag.as_str()));
    quote! {
        #root::Tag::new(#inner)
    }
}
