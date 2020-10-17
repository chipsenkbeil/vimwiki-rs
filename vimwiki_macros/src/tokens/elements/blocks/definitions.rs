use crate::tokens::{root_crate, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_definition_list, DefinitionList<'a>, 'a);
fn tokenize_definition_list(definition_list: &DefinitionList) -> TokenStream {
    let root = root_crate();
    let td = definition_list.iter().map(tokenize_term_and_definitions);
    quote! {
        #root::elements::DefinitionList::from(vec![#(#td),*])
    }
}

fn tokenize_term_and_definitions(
    (term, definitions): &(Term, Vec<Definition>),
) -> TokenStream {
    let root = root_crate();
    let term = do_tokenize!(term);
    let definitions = definitions.iter().map(|x| do_tokenize!(x));
    quote! {
        (#term, vec![#(#definitions),*])
    }
}
