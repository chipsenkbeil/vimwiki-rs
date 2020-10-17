use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::{Located, Position, Region};

impl<T: Tokenize> Tokenize for Located<T> {
    fn tokenize(&self, stream: &mut TokenStream) {
        let root = root_crate();
        let mut element = TokenStream::new();
        self.as_inner().tokenize(&mut element);

        let region = do_tokenize!(self.region);

        let self_stream = quote! {
            #root::elements::Located {
                element: #element,
                region: #region,
            }
        };

        stream.extend(std::iter::once(self_stream))
    }
}

impl_tokenize!(tokenize_region, Region);
fn tokenize_region(region: &Region) -> TokenStream {
    let root = root_crate();
    let start = tokenize_position(&region.start);
    let end = tokenize_position(&region.end);
    quote! {
        #root::elements::Region {
            start: #start,
            end: #end,
        }
    }
}

impl_tokenize!(tokenize_position, Position);
fn tokenize_position(position: &Position) -> TokenStream {
    let root = root_crate();
    let line = position.line;
    let column = position.column;
    quote! {
        #root::elements::Position {
            line: #line,
            column: #column,
        }
    }
}
