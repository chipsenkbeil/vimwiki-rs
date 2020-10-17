use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_header, Header<'a>, 'a);
fn tokenize_header(header: &Header) -> TokenStream {
    let root = root_crate();
    let Header {
        level,
        content,
        centered,
    } = header;
    let content_t = do_tokenize!(&content);
    quote! {
        #root::elements::Header {
            level: #level,
            content: #content_t,
            centered: #centered,
        }
    }
}
