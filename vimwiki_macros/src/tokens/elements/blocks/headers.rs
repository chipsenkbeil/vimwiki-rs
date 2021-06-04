use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::Header;

impl_tokenize!(tokenize_header, Header<'a>, 'a);
fn tokenize_header(ctx: &TokenizeContext, header: &Header) -> TokenStream {
    let root = root_crate();
    let level = header.level;
    let centered = header.centered;
    let content_t = do_tokenize!(ctx, &header.content);
    quote! {
        #root::Header::new(
            #content_t,
            #level,
            #centered,
        )
    }
}
