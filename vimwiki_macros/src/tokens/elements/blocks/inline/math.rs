use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::MathInline;

impl_tokenize!(tokenize_math_inline, MathInline<'a>, 'a);
fn tokenize_math_inline(
    ctx: &TokenizeContext,
    math_inline: &MathInline,
) -> TokenStream {
    let root = root_crate();
    let formula = do_tokenize!(ctx, &math_inline.formula);
    quote! {
        #root::MathInline {
            formula: #formula,
        }
    }
}
