use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::{DecoratedText, DecoratedTextContent, Keyword, Text};

impl_tokenize!(tokenize_text, Text<'a>, 'a);
fn tokenize_text(ctx: &TokenizeContext, text: &Text) -> TokenStream {
    let root = root_crate();
    let inner = do_tokenize!(ctx, text.as_ref());
    quote! {
        #root::Text::new(#inner)
    }
}

impl_tokenize!(tokenize_decorated_text_content, DecoratedTextContent<'a>, 'a);
fn tokenize_decorated_text_content(
    ctx: &TokenizeContext,
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    let root = root_crate();
    match &decorated_text_content {
        DecoratedTextContent::Keyword(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::DecoratedText(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::DecoratedTextContent::DecoratedText(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::DecoratedTextContent::Text(#t) }
        }
    }
}

impl_tokenize!(tokenize_decorated_text, DecoratedText<'a>, 'a);
fn tokenize_decorated_text(
    ctx: &TokenizeContext,
    decorated_text: &DecoratedText,
) -> TokenStream {
    let root = root_crate();

    match decorated_text {
        DecoratedText::Bold(x) => {
            let contents = x.iter().map(|x| do_tokenize!(ctx, x));
            quote! {
                #root::DecoratedText::Bold(
                    ::std::vec![#(#contents),*],
                )
            }
        }
        DecoratedText::Italic(x) => {
            let contents = x.iter().map(|x| do_tokenize!(ctx, x));
            {
                quote! {
                    #root::DecoratedText::Italic(
                        ::std::vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Strikeout(x) => {
            let contents = x.iter().map(|x| do_tokenize!(ctx, x));
            {
                quote! {
                    #root::DecoratedText::Strikeout(
                        ::std::vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Subscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(ctx, x));
            {
                quote! {
                    #root::DecoratedText::Subscript(
                        ::std::vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Superscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(ctx, x));
            {
                quote! {
                    #root::DecoratedText::Superscript(
                        ::std::vec![#(#contents),*],
                    )
                }
            }
        }
    }
}

impl_tokenize!(tokenize_keyword, Keyword);
fn tokenize_keyword(_ctx: &TokenizeContext, keyword: &Keyword) -> TokenStream {
    let root = root_crate();
    match keyword {
        Keyword::Done => {
            quote! { #root::Keyword::Done }
        }
        Keyword::Fixed => {
            quote! { #root::Keyword::Fixed }
        }
        Keyword::Fixme => {
            quote! { #root::Keyword::Fixme }
        }
        Keyword::Started => {
            quote! { #root::Keyword::Started }
        }
        Keyword::Todo => {
            quote! { #root::Keyword::Todo }
        }
        Keyword::Xxx => {
            quote! { #root::Keyword::Xxx }
        }
    }
}
