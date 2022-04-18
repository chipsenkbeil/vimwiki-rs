use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::{
    DefinitionBundle, DefinitionList, DefinitionListValue, Located, Term,
};

impl_tokenize!(tokenize_definition_list, DefinitionList<'a>, 'a);
fn tokenize_definition_list(
    ctx: &TokenizeContext,
    definition_list: &DefinitionList,
) -> TokenStream {
    let root = root_crate();
    let td = definition_list
        .iter()
        .map(|x| tokenize_term_and_definition_bundle(ctx, x));
    quote! {
        <#root::DefinitionList as ::std::iter::FromIterator<
            (
                #root::Located<#root::Term>,
                #root::Located<#root::DefinitionBundle>,
            )
        >>::from_iter(::std::vec![#(#td),*])
    }
}

fn tokenize_term_and_definition_bundle(
    ctx: &TokenizeContext,
    (term, bundle): (&Located<Term>, &Located<DefinitionBundle>),
) -> TokenStream {
    let term = do_tokenize!(ctx, term);
    let bundle = do_tokenize!(ctx, bundle);
    quote! { (#term, #bundle) }
}

impl_tokenize!(tokenize_definition_bundle, DefinitionBundle<'a>, 'a);
fn tokenize_definition_bundle(
    ctx: &TokenizeContext,
    definition_bundle: &DefinitionBundle,
) -> TokenStream {
    let root = root_crate();
    let definitions = definition_bundle.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::DefinitionBundle::new(::std::vec![#(#definitions),*])
    }
}

impl_tokenize!(tokenize_definition_list_value, DefinitionListValue<'a>, 'a);
fn tokenize_definition_list_value(
    ctx: &TokenizeContext,
    definition_list_value: &DefinitionListValue,
) -> TokenStream {
    let root = root_crate();
    let inner = do_tokenize!(ctx, definition_list_value.as_inner());
    quote! {
        #root::DefinitionListValue::new(#inner)
    }
}
