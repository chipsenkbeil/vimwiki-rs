use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_header, Header<'a>, 'a);
fn tokenize_header(header: &Header) -> TokenStream {
    let root = element_path();
    let Header {
        level,
        content,
        centered,
    } = header;
    let content_t = do_tokenize!(&content);
    quote! {
        #root::Header {
            level: #level,
            content: #content_t,
            centered: #centered,
        }
    }
}
