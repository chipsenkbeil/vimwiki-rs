use crate::tokens::{
    utils::root_crate, utils::tokenize_option, Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use std::borrow::Cow;
use vimwiki_core::MathBlock;

impl_tokenize!(tokenize_math_block, MathBlock<'a>, 'a);
fn tokenize_math_block(
    ctx: &TokenizeContext,
    math_block: &MathBlock,
) -> TokenStream {
    let root = root_crate();
    let lines = math_block
        .lines()
        .map(|x| do_tokenize!(ctx, Cow::Borrowed(x)));
    let environment = tokenize_option(
        ctx,
        &math_block.environment().map(Cow::Borrowed),
        |ctx, x| do_tokenize!(ctx, x),
    );
    quote! {
        #root::MathBlock::new(
            ::std::vec![#(#lines),*],
            #environment,
        )
    }
}
