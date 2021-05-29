use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::Placeholder;

impl_tokenize!(tokenize_placeholder, Placeholder<'a>, 'a);
fn tokenize_placeholder(
    ctx: &TokenizeContext,
    placeholder: &Placeholder,
) -> TokenStream {
    let root = root_crate();
    match &placeholder {
        Placeholder::Date(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Placeholder::Date(#t) }
        }
        Placeholder::NoHtml => {
            quote! { #root::Placeholder::NoHtml }
        }
        Placeholder::Other { name, value } => {
            let name_t = do_tokenize!(ctx, &name);
            let value_t = do_tokenize!(ctx, &value);
            quote! {
                #root::Placeholder::Other {
                    name: #name_t,
                    value: #value_t,
                }
            }
        }
        Placeholder::Template(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Placeholder::Template(#t) }
        }
        Placeholder::Title(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Placeholder::Title(#t) }
        }
    }
}
