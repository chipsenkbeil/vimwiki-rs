use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_paragraph, Paragraph<'a>, 'a);
fn tokenize_paragraph(paragraph: &Paragraph) -> TokenStream {
    let root = element_path();
    let content = do_tokenize!(&paragraph.content);
    quote! {
        #root::Paragraph {
            content: #content,
        }
    }
}
