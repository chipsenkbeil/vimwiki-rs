use crate::tokens::{root_crate, utils::tokenize_option, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_table, Table<'a>, 'a);
fn tokenize_table(table: &Table) -> TokenStream {
    let root = root_crate();
    let rows = table.rows.iter().map(|x| do_tokenze!(x));
    let centered = table.centered;
    quote! {
        #root::elements::Table {
            rows: vec![#(#rows),*],
            centered: #centered,
        }
    }
}

impl_tokenize!(tokenize_row, Row<'a>, 'a);
fn tokenize_row(row: &Row) -> TokenStream {
    let root = root_crate();
    match &row {
        Row::Content { cells } => {
            let t = cells.iter().map(|x| do_tokenize!(x));
            quote! { #root::elements::Row::Content { cells: vec![#(#t),*] } }
        }
        Row::Divider => {
            quote! { #root::elements::Row::Divider }
        }
    }
}

impl_tokenize!(tokenize_cell, Cell<'a>, 'a);
fn tokenize_cell(cell: &Cell) -> TokenStream {
    let root = root_crate();
    match &cell {
        Cell::Content(x) => {
            let t = do_tokenze!(&x);
            quote! { #root::elements::Cell::Content(#t) }
        }
        Cell::SpanAbove => {
            quote! { #root::elements::Cell::SpanAbove }
        }
        Cell::SpanLeft => {
            quote! { #root::elements::Cell::SpanLeft }
        }
    }
}
