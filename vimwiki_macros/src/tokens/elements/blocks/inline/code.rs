use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::CodeInline;

impl_tokenize!(tokenize_code_inline, CodeInline<'a>, 'a);
fn tokenize_code_inline(
    ctx: &TokenizeContext,
    code_inline: &CodeInline,
) -> TokenStream {
    let root = root_crate();
    let code = do_tokenize!(ctx, &code_inline.code);
    quote! {
        #root::CodeInline {
            code: #code,
        }
    }
}
