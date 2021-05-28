use crate::tokens::{
    utils::{root_crate, tokenize_cow_str_type, tokenize_hashmap},
    Tokenize, TokenizeContext,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::{Anchor, Description, Link, LinkData};

impl_tokenize!(tokenize_link, Link<'a>, 'a);
fn tokenize_link(ctx: &TokenizeContext, link: &Link) -> TokenStream {
    let root = root_crate();
    match &link {
        Link::Diary { date, data } => {
            let date_t = do_tokenize!(ctx, date);
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::Diary { date: #date_t, data: #data_t })
        }
        Link::IndexedInterWiki { index, data } => {
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::IndexedInterWiki { index: #index, data: #data_t })
        }
        Link::NamedInterWiki { name, data } => {
            let name_t = do_tokenize!(ctx, name);
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::NamedInterWiki { name: #name_t, data: #data_t })
        }
        Link::Raw { data } => {
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::Raw { data: #data_t })
        }
        Link::Transclusion { data } => {
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::Transclusion { data: #data_t })
        }
        Link::Wiki { data } => {
            let data_t = do_tokenize!(ctx, data);
            quote!(#root::Link::Wiki { data: #data_t })
        }
    }
}

impl_tokenize!(tokenize_link_data, LinkData<'a>, 'a);
fn tokenize_link_data(ctx: &TokenizeContext, data: &LinkData) -> TokenStream {
    let root = root_crate();
    let uri_ref_t = do_tokenize!(ctx, data.uri_ref());
    let description_t = if let Some(d) = data.description() {
        let t = tokenize_description(ctx, d);
        quote!(::std::option::Option::Some(#t))
    } else {
        quote!(::std::option::Option::None)
    };
    let properties_t = if let Some(properties) = data.properties() {
        let t = tokenize_hashmap(
            properties,
            tokenize_cow_str_type(),
            tokenize_cow_str_type(),
            |x| do_tokenize!(ctx, x),
            |x| do_tokenize!(ctx, x),
        );
        quote!(::std::option::Option::Some(#t))
    } else {
        quote!(::std::option::Option::None)
    };

    quote!(#root::LinkData::new(#uri_ref_t, #description_t, #properties_t))
}

impl_tokenize!(tokenize_description, Description<'a>, 'a);
fn tokenize_description(
    ctx: &TokenizeContext,
    description: &Description,
) -> TokenStream {
    let root = root_crate();
    match &description {
        Description::Text(x) => {
            let t = do_tokenize!(ctx, &x);
            quote! { #root::Description::Text(#t) }
        }
        Description::TransclusionLink(x) => {
            let t = tokenize_link_data(ctx, &x);
            quote! {
                #root::Description::TransclusionLink(::std::boxed::Box::new(#t))
            }
        }
    }
}

impl_tokenize!(tokenize_anchor, Anchor<'a>, 'a);
fn tokenize_anchor(ctx: &TokenizeContext, anchor: &Anchor) -> TokenStream {
    let root = root_crate();
    let elements = anchor.iter().map(|x| do_tokenize!(ctx, x));
    quote!(#root::Anchor::new(::std::vec![#(#elements),*]))
}
