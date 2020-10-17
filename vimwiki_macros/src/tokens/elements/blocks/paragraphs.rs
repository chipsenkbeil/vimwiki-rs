use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_paragraph, Paragraph<'a>, 'a);
fn tokenize_paragraph(paragraph: &Paragraph) -> TokenStream {
    let root = root_crate();
    let content = do_tokenize!(&paragraph.content);
    quote! {
        #root::elements::Paragraph {
            content: #content,
        }
    }
}
