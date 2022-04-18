use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::{ElementLike, Located, Region};

impl<T: Tokenize> Tokenize for Located<T>
where
    T: ElementLike,
{
    fn tokenize(&self, ctx: &TokenizeContext, stream: &mut TokenStream) {
        let root = root_crate();
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
    let root = root_crate();
    let offset = region.offset();
    let len = region.len();
    quote! {
        #root::Region::new(
            #offset,
            #len,
        )
    }
}
