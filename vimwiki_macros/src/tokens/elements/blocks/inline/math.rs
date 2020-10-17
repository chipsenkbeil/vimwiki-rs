use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_math_inline, MathInline<'a>, 'a);
fn tokenize_math_inline(math_inline: &MathInline) -> TokenStream {
    let root = root_crate();
    let formula = do_tokenize!(&math_inline.formula);
    quote! {
        #root::elements::MathInline {
            formula: #formula,
        }
    }
}
