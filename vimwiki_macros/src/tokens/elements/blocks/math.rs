use crate::tokens::{root_crate, utils::tokenize_option, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_math_block, MathBlock<'a>, 'a);
fn tokenize_math_block(math_block: &MathBlock) -> TokenStream {
    let root = root_crate();
    let lines = math_block.lines.iter().map(|x| do_tokenize!(x));
    let environment =
        tokenize_option(&math_block.environment, |x| do_tokenize!(x));
    quote! {
        #root::elements::MathBlock {
            lines: vec![#(#lines),*],
            environment: #environment,
        }
    }
}
