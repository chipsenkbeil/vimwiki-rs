use crate::tokens::{
    utils::{
        root_crate, tokenize_cow_str_type, tokenize_hashmap, tokenize_option,
    },
    Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::CodeBlock;

impl_tokenize!(tokenize_code_block, CodeBlock<'a>, 'a);
fn tokenize_code_block(
    ctx: &TokenizeContext,
    code_block: &CodeBlock,
) -> TokenStream {
    let root = root_crate();
    let lang =
        tokenize_option(ctx, &code_block.lang, |ctx, x| do_tokenize!(ctx, x));
    let metadata = tokenize_hashmap(
        &code_block.metadata,
        tokenize_cow_str_type(),
        tokenize_cow_str_type(),
        |x| do_tokenize!(ctx, x),
        |x| do_tokenize!(ctx, x),
    );
    let lines = code_block.lines.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::CodeBlock {
            lang: #lang,
            metadata: #metadata,
            lines: ::std::vec![#(#lines),*],
        }
    }
}
