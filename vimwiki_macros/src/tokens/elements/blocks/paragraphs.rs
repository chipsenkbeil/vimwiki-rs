use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_paragraph, Paragraph<'a>, 'a);
fn tokenize_paragraph(
    ctx: &TokenizeContext,
    paragraph: &Paragraph,
) -> TokenStream {
    let root = element_path();
    let content = do_tokenize!(ctx, &paragraph.content);
    quote! {
        #root::Paragraph {
            content: #content,
        }
    }
}
