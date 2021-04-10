use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_text, Text<'a>, 'a);
fn tokenize_text(text: &Text) -> TokenStream {
    let root = element_path();
    let inner = do_tokenize!(text.as_ref());
    quote! {
        #root::Text::new(#inner)
    }
}

impl_tokenize!(tokenize_decorated_text_content, DecoratedTextContent<'a>, 'a);
fn tokenize_decorated_text_content(
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    let root = element_path();
    match &decorated_text_content {
        DecoratedTextContent::Keyword(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::DecoratedText(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::DecoratedTextContent::DecoratedText(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::DecoratedTextContent::Text(#t) }
        }
    }
}

impl_tokenize!(tokenize_decorated_text, DecoratedText<'a>, 'a);
fn tokenize_decorated_text(decorated_text: &DecoratedText) -> TokenStream {
    let root = element_path();

    match decorated_text {
        DecoratedText::Bold(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            quote! {
                #root::DecoratedText::Bold(
                    vec![#(#contents),*],
                )
            }
        }
        DecoratedText::Italic(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::DecoratedText::Italic(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Strikeout(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::DecoratedText::Strikeout(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Subscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::DecoratedText::Subscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Superscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::DecoratedText::Superscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
    }
}

impl_tokenize!(tokenize_keyword, Keyword);
fn tokenize_keyword(keyword: &Keyword) -> TokenStream {
    let root = element_path();
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
