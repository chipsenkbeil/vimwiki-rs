use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_header, Header<'a>, 'a);
fn tokenize_header(ctx: &TokenizeContext, header: &Header) -> TokenStream {
    let root = element_path();
    let Header {
        level,
        content,
        centered,
    } = header;
    let content_t = do_tokenize!(ctx, &content);
    quote! {
        #root::Header {
            level: #level,
            content: #content_t,
            centered: #centered,
        }
    }
}
