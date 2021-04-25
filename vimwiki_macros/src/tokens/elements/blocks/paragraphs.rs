use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::Paragraph;

impl_tokenize!(tokenize_paragraph, Paragraph<'a>, 'a);
fn tokenize_paragraph(
    ctx: &TokenizeContext,
    paragraph: &Paragraph,
) -> TokenStream {
    let root = root_crate();
    let content = do_tokenize!(ctx, &paragraph.content);
    quote! {
        #root::Paragraph {
            content: #content,
        }
    }
}
