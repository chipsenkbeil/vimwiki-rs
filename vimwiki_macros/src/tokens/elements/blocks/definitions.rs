use crate::tokens::{utils::element_path, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_definition_list, DefinitionList<'a>, 'a);
fn tokenize_definition_list(
    ctx: &TokenizeContext,
    definition_list: &DefinitionList,
) -> TokenStream {
    let root = element_path();
    let td = definition_list
        .iter()
        .map(|x| tokenize_term_and_definitions(ctx, x));
    quote! {
        <#root::DefinitionList as ::std::convert::From<
            ::std::vec::Vec<(
                #root::Located<#root::Term>,
                ::std::vec::Vec<#root::Located<#root::Definition>>,
            )>
        >>::from(::std::vec![#(#td),*])
    }
}

fn tokenize_term_and_definitions(
    ctx: &TokenizeContext,
    (term, definitions): (&Located<Term>, &Vec<Located<Definition>>),
) -> TokenStream {
    let term = do_tokenize!(ctx, term);
    let definitions = definitions.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        (#term, ::std::vec![#(#definitions),*])
    }
}

impl_tokenize!(tokenize_definition_list_value, DefinitionListValue<'a>, 'a);
fn tokenize_definition_list_value(
    ctx: &TokenizeContext,
    definition_list_value: &DefinitionListValue,
) -> TokenStream {
    let root = element_path();
    let inner = do_tokenize!(ctx, definition_list_value.as_inner());
    quote! {
        #root::DefinitionListValue::new(#inner)
    }
}
