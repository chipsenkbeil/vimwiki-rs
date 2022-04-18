use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::{BlockElement, InlineBlockElement};

pub mod blockquotes;
pub mod code;
pub mod definitions;
pub mod dividers;
pub mod headers;
pub mod inline;
pub mod lists;
pub mod math;
pub mod paragraphs;
pub mod placeholders;
pub mod tables;

impl_tokenize!(tokenize_block_element, BlockElement<'a>, 'a);
fn tokenize_block_element(
    ctx: &TokenizeContext,
    block_element: &BlockElement,
) -> TokenStream {
    let root = root_crate();
    match block_element {
        BlockElement::Blockquote(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Blockquote(#t) }
        }
        BlockElement::DefinitionList(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::DefinitionList(#t) }
        }
        BlockElement::Divider(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Divider(#t) }
        }
        BlockElement::Header(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Header(#t) }
        }
        BlockElement::List(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::List(#t) }
        }
        BlockElement::MathBlock(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::MathBlock(#t) }
        }
        BlockElement::Paragraph(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Paragraph(#t) }
        }
        BlockElement::Placeholder(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Placeholder(#t) }
        }
        BlockElement::CodeBlock(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::CodeBlock(#t) }
        }
        BlockElement::Table(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::BlockElement::Table(#t) }
        }
    }
}

impl_tokenize!(tokenize_inline_block_element, InlineBlockElement<'a>, 'a);
fn tokenize_inline_block_element(
    ctx: &TokenizeContext,
    inline_block_element: &InlineBlockElement,
) -> TokenStream {
    let root = root_crate();
    match inline_block_element {
        InlineBlockElement::ListItem(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::InlineBlockElement::ListItem(#t) }
        }
        InlineBlockElement::Term(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::InlineBlockElement::Term(#t) }
        }
        InlineBlockElement::Definition(x) => {
            let t = do_tokenize!(ctx, x);
            quote! { #root::InlineBlockElement::Definition(#t) }
        }
    }
}
