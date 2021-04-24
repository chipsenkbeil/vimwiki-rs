use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

pub mod code;
pub mod comments;
pub mod links;
pub mod math;
pub mod tags;
pub mod typefaces;

impl_tokenize!(tokenize_inline_element_container, InlineElementContainer<'a>, 'a);
fn tokenize_inline_element_container(
    ctx: &TokenizeContext,
    inline_element_container: &InlineElementContainer,
) -> TokenStream {
    let root = element_path();
    let elements = inline_element_container
        .elements
        .iter()
        .map(|c| do_tokenize!(ctx, c));
    quote! {
        #root::InlineElementContainer {
            elements: ::std::vec![#(#elements),*],
        }
    }
}

impl_tokenize!(tokenize_inline_element, InlineElement<'a>, 'a);
fn tokenize_inline_element(
    ctx: &TokenizeContext,
    inline_element: &InlineElement,
) -> TokenStream {
    let root = element_path();
    match inline_element {
        InlineElement::Text(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Text(#t) }
        }
        InlineElement::DecoratedText(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::DecoratedText(#t) }
        }
        InlineElement::Keyword(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Keyword(#t) }
        }
        InlineElement::Link(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Link(#t) }
        }
        InlineElement::Tags(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Tags(#t) }
        }
        InlineElement::Code(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Code(#t) }
        }
        InlineElement::Math(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Math(#t) }
        }
        InlineElement::Comment(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::InlineElement::Comment(#t) }
        }
    }
}
