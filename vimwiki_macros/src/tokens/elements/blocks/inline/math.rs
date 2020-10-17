use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_math_inline, MathInline<'a>, 'a);
fn tokenize_math_inline(math_inline: &MathInline) -> TokenStream {
    let root = element_path();
    let formula = do_tokenize!(&math_inline.formula);
    quote! {
        #root::MathInline {
            formula: #formula,
        }
    }
}
