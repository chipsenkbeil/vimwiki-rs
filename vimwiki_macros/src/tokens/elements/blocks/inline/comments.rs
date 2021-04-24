use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_comment, Comment<'a>, 'a);
fn tokenize_comment(ctx: &TokenizeContext, comment: &Comment) -> TokenStream {
    let root = element_path();
    match comment {
        Comment::Line(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Comment::Line(#t) }
        }
        Comment::MultiLine(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Comment::MultiLine(#t) }
        }
    }
}

impl_tokenize!(tokenize_line_comment, LineComment<'a>, 'a);
fn tokenize_line_comment(
    ctx: &TokenizeContext,
    line_comment: &LineComment,
) -> TokenStream {
    let root = element_path();
    let t = do_tokenize!(ctx, &line_comment.0);
    quote! {
        #root::LineComment(#t)
    }
}

impl_tokenize!(tokenize_multi_line_comment, MultiLineComment<'a>, 'a);
fn tokenize_multi_line_comment(
    ctx: &TokenizeContext,
    multi_line_comment: &MultiLineComment,
) -> TokenStream {
    let root = element_path();
    let t = multi_line_comment.0.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::MultiLineComment(::std::vec![#(#t),*])
    }
}
