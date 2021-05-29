use crate::tokens::{Tokenize, TokenizeContext};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use std::collections::HashMap;
use syn::{parse_quote, Ident, Path};

/// Generates a `TokenStream` that provides the root of an import path for
/// the `vimwiki` crate
#[inline]
pub fn root_crate() -> Path {
    get_crate("vimwiki")
        .or_else(|_| get_crate("vimwiki-core"))
        .expect("vimwiki crate exists")
}

fn get_crate(cname: &str) -> syn::Result<Path> {
    crate_name(cname)
        .map(|found_crate| match found_crate {
            FoundCrate::Itself => {
                // Special case for vimwiki integration tests that can be removed when
                // https://github.com/bkchr/proc-macro-crate/issues/10 is resolved
                //
                // NOTE: Must check at compile-time as CARGO_BIN_EXE_ is not
                //       available during runtime
                //parse_quote!(crate)
                parse_quote!(::vimwiki)
            }
            FoundCrate::Name(name) => {
                let crate_ident = Ident::new(&name, Span::mixed_site());
                parse_quote!(::#crate_ident)
            }
        })
        .map_err(|msg| syn::Error::new(Span::mixed_site(), msg))
}

/// Generates a `TokenStream` that provides the path to vendor types in
/// the `vimwiki` crate
#[inline]
pub fn vendor_path() -> TokenStream {
    let root = root_crate();
    quote! { #root::vendor }
}

/// Produces a `TokenStream` for the `String` type
#[inline]
pub fn tokenize_cow_str_type() -> TokenStream {
    quote! { ::std::borrow::Cow<'_, str >}
}

/// Tokenizes a `HashMap<K, V>` where both the key and value implement the
/// `Tokenize` trait. Additionally, uses *kty* and *vty* as the types for
/// the hashmap and *fk* and *fv* as the functions to transform each key
/// and value to `TokenStream`.
pub fn tokenize_hashmap<K: Tokenize, V: Tokenize>(
    m: &HashMap<K, V>,
    kty: TokenStream,
    vty: TokenStream,
    fk: impl Fn(&K) -> TokenStream,
    fv: impl Fn(&V) -> TokenStream,
) -> TokenStream {
    let pairs = m.iter().map(|(k, v)| {
        let tk = fk(k);
        let tv = fv(v);
        quote! { (#tk, #tv) }
    });
    quote! {
        ::std::iter::Iterator::collect::<::std::collections::HashMap<#kty,#vty>>(
            ::std::vec![#(#pairs),*].drain(..),
        )
    }
}

/// Tokenizes an `Option<T>` where the inner type implements the `Tokenize`
/// trait. Additionally, uses the *f* function to transform each inner value
/// into a `TokenStream`.
pub fn tokenize_option<T: Tokenize>(
    ctx: &TokenizeContext,
    o: &Option<T>,
    f: impl Fn(&TokenizeContext, &T) -> TokenStream,
) -> TokenStream {
    if let Some(ref x) = *o {
        let t = f(ctx, x);
        quote! { ::std::option::Option::Some(#t) }
    } else {
        quote! { ::std::option::Option::None }
    }
}
