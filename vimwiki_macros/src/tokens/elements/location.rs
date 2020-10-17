use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::{Located, Position, Region};

impl<T: Tokenize> Tokenize for Located<T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let root = element_path();
        let mut element = TokenStream::new();
        self.as_inner().tokenize(&mut element);

        let region = do_tokenize!(self.region);

        let self_stream = quote! {
            #root::Located {
                element: #element,
                region: #region,
            }
        };

        stream.extend(std::iter::once(self_stream))
    }
}

impl_tokenize!(tokenize_region, Region);
fn tokenize_region(region: &Region) -> TokenStream {
    let root = element_path();
    let start = tokenize_position(&region.start);
    let end = tokenize_position(&region.end);
    quote! {
        #root::Region {
            start: #start,
            end: #end,
        }
    }
}

impl_tokenize!(tokenize_position, Position);
fn tokenize_position(position: &Position) -> TokenStream {
    let root = element_path();
    let line = position.line;
    let column = position.column;
    quote! {
        #root::Position {
            line: #line,
            column: #column,
        }
    }
}
