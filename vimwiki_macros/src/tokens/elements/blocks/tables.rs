use crate::tokens::{utils::element_path, Tokenize};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::elements::*;

impl_tokenize!(tokenize_table, Table<'a>, 'a);
fn tokenize_table(table: &Table) -> TokenStream {
    let root = element_path();
    let rows = table.rows.iter().map(|x| do_tokenize!(x));
    let centered = table.centered;
    quote! {
        #root::Table {
            rows: vec![#(#rows),*],
            centered: #centered,
        }
    }
}

impl_tokenize!(tokenize_row, Row<'a>, 'a);
fn tokenize_row(row: &Row) -> TokenStream {
    let root = element_path();
    match &row {
        Row::Content { cells } => {
            let t = cells.iter().map(|x| do_tokenize!(x));
            quote! { #root::Row::Content { cells: vec![#(#t),*] } }
        }
        Row::Divider => {
            quote! { #root::Row::Divider }
        }
    }
}

impl_tokenize!(tokenize_cell, Cell<'a>, 'a);
fn tokenize_cell(cell: &Cell) -> TokenStream {
    let root = element_path();
    match &cell {
        Cell::Content(x) => {
            let t = do_tokenize!(&x);
            quote! { #root::Cell::Content(#t) }
        }
        Cell::SpanAbove => {
            quote! { #root::Cell::SpanAbove }
        }
        Cell::SpanLeft => {
            quote! { #root::Cell::SpanLeft }
        }
    }
}
