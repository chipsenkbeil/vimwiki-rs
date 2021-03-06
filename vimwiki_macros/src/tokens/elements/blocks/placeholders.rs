use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_placeholder, Placeholder<'a>, 'a);
fn tokenize_placeholder(placeholder: &Placeholder) -> TokenStream {
    let root = element_path();
    match &placeholder {
        Placeholder::Date(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Placeholder::Date(#t) }
        }
        Placeholder::NoHtml => {
            quote! { #root::Placeholder::NoHtml }
        }
        Placeholder::Other { name, value } => {
            let name_t = do_tokenize!(&name);
            let value_t = do_tokenize!(&value);
            quote! {
                #root::Placeholder::Other {
                    name: #name_t,
                    value: #value_t,
                }
            }
        }
        Placeholder::Template(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Placeholder::Template(#t) }
        }
        Placeholder::Title(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Placeholder::Title(#t) }
        }
    }
}
