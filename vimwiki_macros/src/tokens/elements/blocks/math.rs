use crate::tokens::{
    utils::root_crate, utils::tokenize_option, Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::MathBlock;

impl_tokenize!(tokenize_math_block, MathBlock<'a>, 'a);
fn tokenize_math_block(
    ctx: &TokenizeContext,
    math_block: &MathBlock,
) -> TokenStream {
    let root = root_crate();
    let lines = math_block.lines.iter().map(|x| do_tokenize!(ctx, x));
    let environment =
        tokenize_option(ctx, &math_block.environment, |ctx, x| {
            do_tokenize!(ctx, x)
        });
    quote! {
        #root::MathBlock {
            lines: ::std::vec![#(#lines),*],
            environment: #environment,
        }
    }
}
