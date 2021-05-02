use super::inline::inline_element_container;
use crate::lang::{
    elements::{
        Cell, CellPos, CellSpan, ColumnAlign, InlineElementContainer, Located,
        Table,
    },
    parsers::{
        utils::{
            capture, context, deeper, end_of_line_or_input, locate,
            take_line_until1, take_line_while1,
        },
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map, map_parser, opt, value, verify},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, terminated},
};
use std::collections::HashMap;

#[inline]
pub fn table(input: Span) -> IResult<Located<Table>> {
    fn inner(input: Span) -> IResult<Table> {
        // Assume a table is centered if the first row is indented
        let (input, (table_header, centered)) =
            map(pair(space0, deeper(row)), |x| (x.1, !x.0.is_empty()))(input)?;

        // Retrieve remaining rows and prepend the header row
        // NOTE: We must make input shallower because it went one deeper from
        //       the earlier row parse
        let (input, mut rows) = many0(preceded(space0, deeper(row)))(input)?;
        rows.insert(0, table_header);

        // We now need to convert a Vec<Vec<Located<Cell>>> into a
        // HashMap<CellPos, Located<Cell>> by using the ordering of the vecs
        // to build out the position
        let cells: HashMap<CellPos, Located<Cell>> = rows
            .into_iter()
            .enumerate()
            .flat_map(|(row_idx, row)| {
                row.into_iter()
                    .enumerate()
                    .map(|(col_idx, cell)| {
                        (CellPos::new(row_idx, col_idx), cell)
                    })
                    .collect::<Vec<(CellPos, Located<Cell>)>>()
            })
            .collect();

        Ok((input, Table::new(cells, centered)))
    }

    // Parse the table and make sure it isn't comprised entirely of divider rows
    context(
        "Table",
        locate(capture(verify(inner, |t| {
            !t.rows().all(|r| r.is_divider_row())
        }))),
    )(input)
}

#[inline]
fn row(input: Span) -> IResult<Vec<Located<Cell>>> {
    context(
        "Row",
        terminated(
            delimited(
                char('|'),
                separated_list1(char('|'), deeper(cell)),
                char('|'),
            ),
            end_of_line_or_input,
        ),
    )(input)
}

#[inline]
fn column_align(input: Span) -> IResult<ColumnAlign> {
    let (input, maybe_start_colon) = opt(char(':'))(input)?;
    let (input, _) = take_line_while1(char('-'))(input)?;
    let (input, maybe_end_colon) = opt(char(':'))(input)?;

    let col = match (maybe_start_colon.is_some(), maybe_end_colon.is_some()) {
        (true, true) => ColumnAlign::Center,
        (false, true) => ColumnAlign::Right,
        (true, false) => ColumnAlign::Left,
        _ => ColumnAlign::default(),
    };

    Ok((input, col))
}

#[inline]
fn cell(input: Span) -> IResult<Located<Cell>> {
    fn inner(input: Span) -> IResult<Cell> {
        alt((
            map(cell_span_above, Cell::Span),
            map(cell_span_left, Cell::Span),
            map(column_align, Cell::Align),
            map(
                map_parser(take_line_until1("|"), inline_element_container),
                |l: Located<InlineElementContainer>| {
                    Cell::Content(l.into_inner())
                },
            ),
        ))(input)
    }

    context("Cell", locate(capture(inner)))(input)
}

#[inline]
fn cell_span_left(input: Span) -> IResult<CellSpan> {
    value(CellSpan::FromLeft, delimited(space0, tag(">"), space0))(input)
}

#[inline]
fn cell_span_above(input: Span) -> IResult<CellSpan> {
    value(CellSpan::FromAbove, delimited(space0, tag("\\/"), space0))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{InlineElement, Link, Text, WikiLink};
    use indoc::indoc;
    use std::path::PathBuf;

    fn check_cell_text_value(cell: &Cell, value: &str) {
        check_cell_value(cell, |c| {
            assert_eq!(c, &InlineElement::Text(Text::from(value)));
        });
    }

    fn check_cell_value(cell: &Cell, f: impl Fn(&InlineElement)) {
        match cell {
            Cell::Content(x) => {
                assert_eq!(
                    x.elements.len(),
                    1,
                    "Unexpected number of inline elements in cell"
                );
                f(x.elements[0].as_inner());
            }
            x => panic!("Unexpected cell: {:?}", x),
        }
    }

    #[test]
    fn table_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_not_starting_with_pipe() {
        let input = Span::from(indoc! {"
        name|age|
        |---|---|
        |abc|012|
        |def|045|
        "});
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_not_ending_with_pipe() {
        let input = Span::from(indoc! {"
        |name|age
        |---|---|
        |abc|012|
        |def|045|
        "});
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_no_content_row_found() {
        let input = Span::from("|---------|");
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_properly_adjust_depth_for_rows_and_cells() {
        let input = Span::from(indoc! {"
        |one|two|
        |---|---|
        |abc|def|
        |ghi|jkl|
        "});

        let (_, tbl) = table(input).unwrap();
        assert_eq!(tbl.depth(), 0, "Table depth was at wrong level");
        for row in tbl.rows.iter() {
            assert_eq!(row.depth(), 1, "Row depth was at wrong level");
            if let Row::Content { cells } = row.as_inner() {
                for cell in cells.iter() {
                    assert_eq!(
                        cell.depth(),
                        2,
                        "Cell depth was at wrong level"
                    );
                }
            }
        }
    }

    #[test]
    fn table_should_succeed_if_uneven_columns_found() {
        let input = Span::from(indoc! {"
        |name| age|
        |----|----|
        |abcd|1111|
        |efgh|2222|3333|
        |ijkl|4444|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "name");

        let cell = t.get_cell(0, 1).unwrap().as_inner();
        check_cell_text_value(cell, " age");

        assert_eq!(
            t.rows[1].as_inner(),
            &Row::Divider {
                columns: vec![ColumnAlign::Left, ColumnAlign::Left]
            }
        );

        let cell = t.get_cell(2, 0).unwrap().as_inner();
        check_cell_text_value(cell, "abcd");

        let cell = t.get_cell(2, 1).unwrap().as_inner();
        check_cell_text_value(cell, "1111");

        let cell = t.get_cell(3, 0).unwrap().as_inner();
        check_cell_text_value(cell, "efgh");

        let cell = t.get_cell(3, 1).unwrap().as_inner();
        check_cell_text_value(cell, "2222");

        let cell = t.get_cell(3, 2).unwrap().as_inner();
        check_cell_text_value(cell, "3333");

        let cell = t.get_cell(4, 0).unwrap().as_inner();
        check_cell_text_value(cell, "ijkl");

        let cell = t.get_cell(4, 1).unwrap().as_inner();
        check_cell_text_value(cell, "4444");
    }

    #[test]
    fn table_should_support_single_row_with_single_cell() {
        let input = Span::from("|value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_single_row_with_multiple_cells() {
        let input = Span::from("|value1|value2|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        let cell = t.get_cell(0, 1).unwrap().as_inner();
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_single_cells() {
        let input = Span::from(indoc! {"
        |value1|
        |value2|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        let cell = t.get_cell(1, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_multiple_cells() {
        let input = Span::from(indoc! {"
        |value1|value2|
        |value3|value4|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        let cell = t.get_cell(0, 1).unwrap().as_inner();
        check_cell_text_value(cell, "value2");

        let cell = t.get_cell(1, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value3");

        let cell = t.get_cell(1, 1).unwrap().as_inner();
        check_cell_text_value(cell, "value4");
    }

    #[test]
    fn table_should_support_row_and_divider_with_single_cell() {
        let input = Span::from(indoc! {"
        |value1|
        |------|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.is_empty(),
            "Did not consume table: '{}'",
            input.as_unsafe_remaining_str()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        assert_eq!(
            t.rows[1].as_inner(),
            &Row::Divider {
                columns: vec![ColumnAlign::Left]
            }
        );
    }

    #[test]
    fn table_should_support_row_and_divider_with_multiple_cells() {
        let input = Span::from(indoc! {"
        |value1|value2|
        |------|------|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.is_empty(),
            "Did not consume table: '{}'",
            input.as_unsafe_remaining_str()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        let cell = t.get_cell(0, 1).unwrap().as_inner();
        check_cell_text_value(cell, "value2");

        assert_eq!(
            t.rows[1].as_inner(),
            &Row::Divider {
                columns: vec![ColumnAlign::Left, ColumnAlign::Left]
            }
        );
    }

    #[test]
    fn table_should_support_row_and_divider_with_different_column_alignments() {
        let input = Span::from(indoc! {"
        |value1|value2|value3|value4|
        |------|:-----|-----:|:----:|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.is_empty(),
            "Did not consume table: '{}'",
            input.as_unsafe_remaining_str()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");

        assert_eq!(
            t.rows[1].as_inner(),
            &Row::Divider {
                columns: vec![
                    ColumnAlign::Left,
                    ColumnAlign::Left,
                    ColumnAlign::Right,
                    ColumnAlign::Center
                ]
            }
        );
    }

    #[test]
    fn table_should_support_span_left_cell() {
        let input = Span::from("|>|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        assert_eq!(cell, &Cell::SpanLeft);
    }

    #[test]
    fn table_should_support_span_above_cell() {
        let input = Span::from(r"|\/|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        assert_eq!(cell, &Cell::SpanAbove);
    }

    #[test]
    fn table_should_support_centering_through_indentation() {
        let input = Span::from(" |value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(t.centered, "Table unexpectedly not centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_inline_content_in_cells() {
        let input = Span::from("|[[some link]]|");
        let (input, t) = table(input).unwrap();
        assert!(input.is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = t.get_cell(0, 0).unwrap().as_inner();
        check_cell_value(cell, |c| {
            assert_eq!(
                c,
                &InlineElement::Link(Link::from(WikiLink::from(
                    PathBuf::from("some link")
                )))
            );
        });
    }
}
