use crate::tokens::{utils::vendor_path, Tokenize};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{borrow::Cow, path::Path};
use vimwiki::vendor::{chrono::NaiveDate, uriparse::URI};

// Implement primitives that already implement ToTokens via quote crate
impl_tokenize!(bool);
impl_tokenize!(i32);
impl_tokenize!(u32);
impl_tokenize!(f32);
impl_tokenize!(f64);
impl_tokenize!(str);
impl_tokenize!(String);

impl<'a, T: ?Sized + ToOwned + ToTokens> Tokenize for Cow<'a, T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        self.to_tokens(stream)
    }
}

// Implement primitives that need custom logic using a manual tokenize function
impl_tokenize!(tokenize_naive_date, NaiveDate);
impl_tokenize!(tokenize_uri, URI<'a>, 'a);
impl_tokenize!(tokenize_path, Path);

fn tokenize_naive_date(naive_date: &NaiveDate) -> TokenStream {
    use vimwiki::vendor::chrono::Datelike;
    let root = vendor_path();
    let year = naive_date.year();
    let month = naive_date.month();
    let day = naive_date.day();
    quote! { #root::chrono::NaiveDate::from_ymd(#year, #month, #day) }
}

fn tokenize_uri(uri: &URI) -> TokenStream {
    let root = vendor_path();
    let uri_string = uri.to_string();
    quote! {
        {
            use std::convert::TryFrom;
            #root::uriparse::URI::try_from(#uri_string)
                .expect("Failed to parse URI").into_owned()
        }
    }
}

fn tokenize_path(path: &Path) -> TokenStream {
    // TODO: Support cases where pathbuf cannot be converted back to Rust str
    let t = path.to_str().expect("Path cannot be converted to &str");
    quote! {
        std::path::Path::new(#t)
    }
}
