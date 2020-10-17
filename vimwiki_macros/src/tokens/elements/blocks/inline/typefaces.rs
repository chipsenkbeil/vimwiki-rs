use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_text, Text<'a>, 'a);
fn tokenize_text(text: &Text) -> TokenStream {
    let root = root_crate();
    let inner = do_tokenize!(text.as_ref());
    quote! {
        #root::elements::Text::new(#inner)
    }
}

impl_tokenize!(tokenize_decorated_text_content, DecoratedTextContent<'a>, 'a);
fn tokenize_decorated_text_content(
    decorated_text_content: &DecoratedTextContent,
) -> TokenStream {
    let root = root_crate();
    match &decorated_text_content {
        DecoratedTextContent::Keyword(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::DecoratedTextContent::Keyword(#t) }
        }
        DecoratedTextContent::Link(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::DecoratedTextContent::Link(#t) }
        }
        DecoratedTextContent::DecoratedText(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::DecoratedTextContent::DecoratedText(#t) }
        }
        DecoratedTextContent::Text(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::elements::DecoratedTextContent::Text(#t) }
        }
    }
}

impl_tokenize!(tokenize_decorated_text, DecoratedText<'a>, 'a);
fn tokenize_decorated_text(decorated_text: &DecoratedText) -> TokenStream {
    let root = root_crate();

    match decorated_text {
        DecoratedText::Bold(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            quote! {
                #root::elements::DecoratedText::Bold(
                    vec![#(#contents),*],
                )
            }
        }
        DecoratedText::BoldItalic(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::elements::DecoratedText::BoldItalic(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Italic(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::elements::DecoratedText::Italic(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Strikeout(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::elements::DecoratedText::Strikeout(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Subscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::elements::DecoratedText::Subscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
        DecoratedText::Superscript(x) => {
            let contents = x.iter().map(|x| do_tokenize!(x));
            {
                quote! {
                    #root::elements::DecoratedText::Superscript(
                        vec![#(#contents),*],
                    )
                }
            }
        }
    }
}

impl_tokenize!(tokenize_keyword, Keyword);
fn tokenize_keyword(keyword: &Keyword) -> TokenStream {
    let root = root_crate();
    match keyword {
        Keyword::DONE => {
            quote! { #root::elements::Keyword::DONE }
        }
        Keyword::FIXED => {
            quote! { #root::elements::Keyword::FIXED }
        }
        Keyword::FIXME => {
            quote! { #root::elements::Keyword::FIXME }
        }
        Keyword::STARTED => {
            quote! { #root::elements::Keyword::STARTED }
        }
        Keyword::TODO => {
            quote! { #root::elements::Keyword::TODO }
        }
        Keyword::XXX => {
            quote! { #root::elements::Keyword::XXX }
        }
    }
}
