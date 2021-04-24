use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::{Located, Region};

impl<T: Tokenize> Tokenize for Located<T> {
    fn tokenize(&self, ctx: &TokenizeContext, stream: &mut TokenStream) {
        let root = element_path();
        let mut element = TokenStream::new();
        self.as_inner().tokenize(ctx, &mut element);

        let region = do_tokenize!(ctx, self.region());

        let self_stream = quote! {
            #root::Located::new(
                #element,
                #region,
            )
        };

        stream.extend(std::iter::once(self_stream))
    }
}

impl_tokenize!(tokenize_region, Region);
fn tokenize_region(_ctx: &TokenizeContext, region: &Region) -> TokenStream {
    let root = element_path();
    let offset = region.offset();
    let len = region.len();
    quote! {
        #root::Region::new(
            #offset,
            #len,
        )
    }
}
