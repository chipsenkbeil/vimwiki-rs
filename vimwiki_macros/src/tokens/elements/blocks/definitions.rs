use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::{
    DefinitionBundle, DefinitionList, DefinitionListValue, TermAndDefinitions,
};

impl_tokenize!(tokenize_definition_list, DefinitionList<'a>, 'a);
fn tokenize_definition_list(
    ctx: &TokenizeContext,
    definition_list: &DefinitionList,
) -> TokenStream {
    let root = root_crate();
    let td = definition_list.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::DefinitionList::new(::std::vec![#(#td),*])
    }
}

impl_tokenize!(tokenize_term_and_definitions, TermAndDefinitions<'a>, 'a);
fn tokenize_term_and_definitions(
    ctx: &TokenizeContext,
    term_and_definitions: &TermAndDefinitions,
) -> TokenStream {
    let root = root_crate();
    let term = do_tokenize!(ctx, term_and_definitions.term);
    let definitions = do_tokenize!(ctx, term_and_definitions.definitions);
    quote! {
        #root::TermAndDefinitions::new(#term, #definitions)
    }
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
