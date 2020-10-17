use proc_macro2::TokenStream;
use quote::quote;

/// Generates a `TokenStream` that provides the root of an import path for
/// the `vimwiki` crate
#[inline]
fn root_crate() -> TokenStream {
    // TODO: Support detecting if we're within the vimwiki crate
    //       (for unit tests only, not integration tests)
    quote! { ::vimwiki }
}

/// Tokenize a value into a stream of tokens.
pub trait Tokenize {
    /// Inject self into a [`TokenStream`].
    fn tokenize(&self, stream: &mut TokenStream);
}

/// Implements `Tokenize` using either a type that implements `ToTokens` or
/// a custom tokenize function and optional generic type information
macro_rules! impl_tokenize {
    ($type:ty) => {
        impl Tokenize for $type {
            fn tokenize(&self, stream: &mut TokenStream) {
                self.to_tokens(stream)
            }
        }
    };
    ($tokenizer:ident, $type:ty) => {
        impl Tokenize for $type {
            fn tokenize(&self, stream: &mut TokenStream) {
                stream.extend(std::iter::once($tokenizer(self)))
            }
        }
    };
    ($tokenizer:ident, $type:ty, $($type_args:tt)+) => {
        impl<$($type_args)+> Tokenize for $type {
            fn tokenize(&self, stream: &mut TokenStream) {
                stream.extend(std::iter::once($tokenizer(self)))
            }
        }
    };
}

/// Transforms expression implementing `Tokenize` into a `TokenStream`
macro_rules! do_tokenize {
    ($value:expr) => {{
        let mut stream = ::proc_macro2::TokenStream::new();
        $value.tokenize(&mut stream);
        stream
    }};
}

mod elements;
mod primitives;
mod utils;
