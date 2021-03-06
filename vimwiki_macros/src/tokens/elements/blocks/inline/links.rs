use crate::tokens::{
    utils::{
        element_path, tokenize_cow_str_type, tokenize_hashmap, tokenize_option,
    },
    Tokenize,
};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_link, Link<'a>, 'a);
fn tokenize_link(link: &Link) -> TokenStream {
    let root = element_path();
    match &link {
        Link::Diary(x) => {
            let t = tokenize_diary_link(&x);
            quote! { #root::Link::Diary(#t) }
        }
        Link::ExternalFile(x) => {
            let t = tokenize_external_file_link(&x);
            quote! { #root::Link::ExternalFile(#t) }
        }
        Link::InterWiki(x) => {
            let t = tokenize_inter_wiki_link(&x);
            quote! { #root::Link::InterWiki(#t) }
        }
        Link::Raw(x) => {
            let t = tokenize_raw_link(&x);
            quote! { #root::Link::Raw(#t) }
        }
        Link::Transclusion(x) => {
            let t = tokenize_transclusion_link(&x);
            quote! { #root::Link::Transclusion(#t) }
        }
        Link::Wiki(x) => {
            let t = tokenize_wiki_link(&x);
            quote! { #root::Link::Wiki(#t) }
        }
    }
}

impl_tokenize!(tokenize_diary_link, DiaryLink<'a>, 'a);
fn tokenize_diary_link(diary_link: &DiaryLink) -> TokenStream {
    let root = element_path();
    let date = do_tokenize!(&diary_link.date);
    let description =
        tokenize_option(&diary_link.description, tokenize_description);
    let anchor = tokenize_option(&diary_link.anchor, tokenize_anchor);
    quote! {
        #root::DiaryLink {
            date: #date,
            description: #description,
            anchor: #anchor,
        }
    }
}

impl_tokenize!(tokenize_external_file_link, ExternalFileLink<'a>, 'a);
fn tokenize_external_file_link(
    external_file_link: &ExternalFileLink,
) -> TokenStream {
    let root = element_path();
    let scheme = tokenize_external_file_link_scheme(&external_file_link.scheme);
    let path = do_tokenize!(&external_file_link.path);
    let description =
        tokenize_option(&external_file_link.description, tokenize_description);
    quote! {
        #root::ExternalFileLink {
            scheme: #scheme,
            path: #path,
            description: #description,
        }
    }
}

impl_tokenize!(tokenize_external_file_link_scheme, ExternalFileLinkScheme);
fn tokenize_external_file_link_scheme(
    external_file_link_scheme: &ExternalFileLinkScheme,
) -> TokenStream {
    let root = element_path();
    match &external_file_link_scheme {
        ExternalFileLinkScheme::Absolute => {
            quote! { #root::ExternalFileLinkScheme::Absolute }
        }
        ExternalFileLinkScheme::File => {
            quote! { #root::ExternalFileLinkScheme::File }
        }
        ExternalFileLinkScheme::Local => {
            quote! { #root::ExternalFileLinkScheme::Local }
        }
    }
}

impl_tokenize!(tokenize_raw_link, RawLink<'a>, 'a);
fn tokenize_raw_link(raw_link: &RawLink) -> TokenStream {
    let root = element_path();
    let uri = do_tokenize!(&raw_link.uri);
    quote! {
        #root::RawLink {
            uri: #uri,
        }
    }
}

impl_tokenize!(tokenize_transclusion_link, TransclusionLink<'a>, 'a);
fn tokenize_transclusion_link(
    transclusion_link: &TransclusionLink,
) -> TokenStream {
    let root = element_path();
    let uri = do_tokenize!(&transclusion_link.uri);
    let description =
        tokenize_option(&transclusion_link.description, tokenize_description);
    let properties = tokenize_hashmap(
        &transclusion_link.properties,
        tokenize_cow_str_type(),
        tokenize_cow_str_type(),
        |x| do_tokenize!(x),
        |x| do_tokenize!(x),
    );
    quote! {
        #root::TransclusionLink {
            uri: #uri,
            description: #description,
            properties: #properties,
        }
    }
}

impl_tokenize!(tokenize_wiki_link, WikiLink<'a>, 'a);
fn tokenize_wiki_link(wiki_link: &WikiLink) -> TokenStream {
    let root = element_path();
    let path = do_tokenize!(&wiki_link.path);
    let description =
        tokenize_option(&wiki_link.description, tokenize_description);
    let anchor = tokenize_option(&wiki_link.anchor, tokenize_anchor);
    quote! {
        #root::WikiLink {
            path: #path,
            description: #description,
            anchor: #anchor,
        }
    }
}

impl_tokenize!(tokenize_indexed_inter_wiki_link, IndexedInterWikiLink<'a>, 'a);
fn tokenize_inter_wiki_link(inter_wiki_link: &InterWikiLink) -> TokenStream {
    let root = element_path();
    match &inter_wiki_link {
        InterWikiLink::Indexed(x) => {
            let t = tokenize_indexed_inter_wiki_link(&x);
            quote! { #root::InterWikiLink::Indexed(#t) }
        }
        InterWikiLink::Named(x) => {
            let t = tokenize_named_inter_wiki_link(&x);
            quote! { #root::InterWikiLink::Named(#t) }
        }
    }
}

impl_tokenize!(tokenize_inter_wiki_link, InterWikiLink<'a>, 'a);
fn tokenize_indexed_inter_wiki_link(
    indexed_inter_wiki_link: &IndexedInterWikiLink,
) -> TokenStream {
    let root = element_path();
    let index = indexed_inter_wiki_link.index;
    let link = tokenize_wiki_link(&indexed_inter_wiki_link.link);
    quote! {
        #root::IndexedInterWikiLink {
            index: #index,
            link: #link,
        }
    }
}

impl_tokenize!(tokenize_named_inter_wiki_link, NamedInterWikiLink<'a>, 'a);
fn tokenize_named_inter_wiki_link(
    named_inter_wiki_link: &NamedInterWikiLink,
) -> TokenStream {
    let root = element_path();
    let name = do_tokenize!(&named_inter_wiki_link.name);
    let link = tokenize_wiki_link(&named_inter_wiki_link.link);
    quote! {
        #root::NamedInterWikiLink {
            name: #name,
            link: #link,
        }
    }
}

impl_tokenize!(tokenize_description, Description<'a>, 'a);
fn tokenize_description(description: &Description) -> TokenStream {
    let root = element_path();
    match &description {
        Description::Text(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Description::Text(#t) }
        }
        Description::URI(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Description::URI(#t) }
        }
    }
}

impl_tokenize!(tokenize_anchor, Anchor<'a>, 'a);
fn tokenize_anchor(anchor: &Anchor) -> TokenStream {
    let root = element_path();
    let elements = anchor.elements.iter().map(|x| do_tokenize!(x));
    quote! {
        #root::Anchor {
            elements: vec![#(#elements),*],
        }
    }
}
