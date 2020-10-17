use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_definition_list, DefinitionList<'a>, 'a);
fn tokenize_definition_list(definition_list: &DefinitionList) -> TokenStream {
    let root = element_path();
    let td = definition_list.iter().map(tokenize_term_and_definitions);
    quote! {
        #root::DefinitionList::from(vec![#(#td),*])
    }
}

fn tokenize_term_and_definitions(
    (term, definitions): (&Term, &Vec<Definition>),
) -> TokenStream {
    let term = do_tokenize!(term);
    let definitions = definitions.iter().map(|x| do_tokenize!(x));
    quote! {
        (#term, vec![#(#definitions),*])
    }
}

impl_tokenize!(tokenize_definition_list_value, DefinitionListValue<'a>, 'a);
fn tokenize_definition_list_value(
    definition_list_value: &DefinitionListValue,
) -> TokenStream {
    let root = element_path();
    let inner = do_tokenize!(definition_list_value.as_inner());
    quote! {
        #root::DefinitionListValue::new(#inner)
    }
}
