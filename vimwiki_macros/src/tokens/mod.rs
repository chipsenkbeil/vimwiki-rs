use proc_macro2::TokenStream;

/// Represents some context to apply when tokenizing a stream
#[derive(Default)]
pub struct TokenizeContext {
    pub verbatim: bool,
}

/// Tokenize a value into a stream of tokens.
pub trait Tokenize {
    /// Inject self into a [`TokenStream`].
    fn tokenize(&self, ctx: &TokenizeContext, stream: &mut TokenStream);
}

/// Implements `Tokenize` using either a type that implements `ToTokens` or
/// a custom tokenize function and optional generic type information
macro_rules! impl_tokenize {
    ($type:ty) => {
        impl Tokenize for $type {
            fn tokenize(&self, _ctx: &crate::TokenizeContext, stream: &mut TokenStream) {
                self.to_tokens(stream)
            }
        }
    };
    ($tokenizer:ident, $type:ty) => {
        impl Tokenize for $type {
            fn tokenize(&self, ctx: &crate::TokenizeContext, stream: &mut TokenStream) {
                stream.extend(std::iter::once($tokenizer(ctx, self)))
            }
        }
    };
    ($tokenizer:ident, $type:ty, $($type_args:tt)+) => {
        impl<$($type_args)+> Tokenize for $type {
            fn tokenize(&self, ctx: &crate::TokenizeContext, stream: &mut TokenStream) {
                stream.extend(std::iter::once($tokenizer(ctx, self)))
            }
        }
    };
}

/// Transforms expression implementing `Tokenize` into a `TokenStream`
macro_rules! do_tokenize {
    ($ctx:expr, $value:expr) => {{
        let mut stream = ::proc_macro2::TokenStream::new();
        $value.tokenize($ctx, &mut stream);
        stream
    }};
}

pub mod elements;
pub mod primitives;
pub mod utils;
