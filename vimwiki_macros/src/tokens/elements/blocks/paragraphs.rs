use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::Paragraph;

impl_tokenize!(tokenize_paragraph, Paragraph<'a>, 'a);
fn tokenize_paragraph(
    ctx: &TokenizeContext,
    paragraph: &Paragraph,
) -> TokenStream {
    let root = root_crate();
    let lines = paragraph.lines.iter().map(|line| do_tokenize!(ctx, line));
    quote! {
        #root::Paragraph::new(::std::vec![#(#lines),*])
    }
}
