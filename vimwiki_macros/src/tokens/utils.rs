use crate::tokens::Tokenize;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;

/// Generates a `TokenStream` that provides the root of an import path for
/// the `vimwiki` crate
#[inline]
pub fn root_crate() -> TokenStream {
    // TODO: Support detecting if we're within the vimwiki crate
    //       (for unit tests only, not integration tests)
    quote! { ::vimwiki }
}

/// Generates a `TokenStream` that provides the path to element types in
/// the `vimwiki` crate
#[inline]
pub fn element_path() -> TokenStream {
    let root = root_crate();
    quote! { #root::elements }
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
    quote! { Cow<'_, str >}
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
        vec![#(#pairs),*].drain(..).collect::<std::collections::HashMap<#kty,#vty>>()
    }
}

/// Tokenizes an `Option<T>` where the inner type implements the `Tokenize`
/// trait. Additionally, uses the *f* function to transform each inner value
/// into a `TokenStream`.
pub fn tokenize_option<T: Tokenize>(
    o: &Option<T>,
    f: impl Fn(&T) -> TokenStream,
) -> TokenStream {
    if let Some(ref x) = *o {
        let t = f(x);
        quote! { Some(#t) }
    } else {
        quote! { None }
    }
}
