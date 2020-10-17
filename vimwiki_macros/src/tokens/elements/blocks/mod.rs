use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

mod blockquotes;
mod definitions;
mod dividers;
mod headers;
mod inline;
mod lists;
mod math;
mod paragraphs;
mod placeholders;
mod preformatted;
mod tables;

impl_tokenize!(tokenize_block_element, BlockElement<'a>, 'a);
fn tokenize_block_element(block_element: &BlockElement) -> TokenStream {
    let root = root_crate();
    match block_element {
        BlockElement::Blockquote(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Blockquote(#t) }
        }
        BlockElement::DefinitionList(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::DefinitionList(#t) }
        }
        BlockElement::Divider(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Divider(#t) }
        }
        BlockElement::Header(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Header(#t) }
        }
        BlockElement::List(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::List(#t) }
        }
        BlockElement::Math(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Math(#t) }
        }
        BlockElement::Paragraph(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Paragraph(#t) }
        }
        BlockElement::Placeholder(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Placeholder(#t) }
        }
        BlockElement::PreformattedText(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::PreformattedText(#t) }
        }
        BlockElement::Table(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::BlockElement::Table(#t) }
        }
    }
}
