use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

pub mod blockquotes;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod inline;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod preformatted;
pub mod tables;

impl_tokenize!(tokenize_block_element, BlockElement<'a>, 'a);
fn tokenize_block_element(block_element: &BlockElement) -> TokenStream {
    let root = element_path();
    match block_element {
        BlockElement::Blockquote(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Blockquote(#t) }
        }
        BlockElement::DefinitionList(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::DefinitionList(#t) }
        }
        BlockElement::Divider(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Divider(#t) }
        }
        BlockElement::Header(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Header(#t) }
        }
        BlockElement::List(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::List(#t) }
        }
        BlockElement::Math(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Math(#t) }
        }
        BlockElement::Paragraph(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Paragraph(#t) }
        }
        BlockElement::Placeholder(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Placeholder(#t) }
        }
        BlockElement::PreformattedText(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::PreformattedText(#t) }
        }
        BlockElement::Table(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::BlockElement::Table(#t) }
        }
    }
}
