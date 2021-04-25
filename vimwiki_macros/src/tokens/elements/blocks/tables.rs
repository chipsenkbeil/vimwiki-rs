use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki::{Cell, ColumnAlign, Row, Table};

impl_tokenize!(tokenize_table, Table<'a>, 'a);
fn tokenize_table(ctx: &TokenizeContext, table: &Table) -> TokenStream {
    let root = root_crate();
    let rows = table.rows.iter().map(|x| do_tokenize!(ctx, x));
    let centered = table.centered;
    quote! {
        #root::Table {
            rows: ::std::vec![#(#rows),*],
            centered: #centered,
        }
    }
}

impl_tokenize!(tokenize_row, Row<'a>, 'a);
fn tokenize_row(ctx: &TokenizeContext, row: &Row) -> TokenStream {
    let root = root_crate();
    match &row {
        Row::Content { cells } => {
            let t = cells.iter().map(|x| do_tokenize!(ctx, x));
            quote! { #root::Row::Content { cells: ::std::vec![#(#t),*] } }
        }
        Row::Divider { columns } => {
            let t = columns.iter().map(|x| do_tokenize!(ctx, x));
            quote! { #root::Row::Divider { columns: ::std::vec![#(#t),*] } }
        }
    }
}

impl_tokenize!(tokenize_cell, Cell<'a>, 'a);
fn tokenize_cell(ctx: &TokenizeContext, cell: &Cell) -> TokenStream {
    let root = root_crate();
    match &cell {
        Cell::Content(x) => {
            let t = do_tokenize!(ctx, &x);
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

impl_tokenize!(tokenize_column_align, ColumnAlign);
fn tokenize_column_align(
    _ctx: &TokenizeContext,
    column_align: &ColumnAlign,
) -> TokenStream {
    let root = root_crate();
    match column_align {
        ColumnAlign::Left => {
            quote! { #root::ColumnAlign::Left }
        }
        ColumnAlign::Center => {
            quote! { #root::ColumnAlign::Center }
        }
        ColumnAlign::Right => {
            quote! { #root::ColumnAlign::Right }
        }
    }
}
