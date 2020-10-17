use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

mod code;
mod comments;
mod links;
mod math;
mod tags;
mod typefaces;

impl_tokenize!(tokenize_inline_element_container, InlineElementContainer<'a>, 'a);
fn tokenize_inline_element_container(
    inline_element_container: &InlineElementContainer,
) -> TokenStream {
    let root = root_crate();
    let elements = inline_element_container
        .elements
        .iter()
        .map(|c| do_tokenize!(c));
    quote! {
        #root::elements::InlineElementContainer {
            elements: vec![#(#elements),*],
        }
    }
}

impl_tokenize!(tokenize_inline_element, InlineElement<'a>, 'a);
fn tokenize_inline_element(inline_element: &InlineElement) -> TokenStream {
    let root = root_crate();
    match inline_element {
        InlineElement::Text(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Text(#t) }
        }
        InlineElement::DecoratedText(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::DecoratedText(#t) }
        }
        InlineElement::Keyword(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Keyword(#t) }
        }
        InlineElement::Link(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Link(#t) }
        }
        InlineElement::Tags(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Tags(#t) }
        }
        InlineElement::Code(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Code(#t) }
        }
        InlineElement::Math(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::InlineElement::Math(#t) }
        }
    }
}
