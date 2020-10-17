use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_code_inline, CodeInline<'a>, 'a);
fn tokenize_code_inline(code_inline: &CodeInline) -> TokenStream {
    let root = root_crate();
    let code = do_tokenize!(&code_inline.code);
    quote! {
        #root::elements::CodeInline {
            code: #code,
        }
    }
}
