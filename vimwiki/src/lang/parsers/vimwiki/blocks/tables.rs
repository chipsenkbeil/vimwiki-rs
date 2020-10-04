use super::{
    elements::{Cell, Row, Table},
    inline::inline_element_container,
    utils::{context, end_of_line_or_input, le, take_line_while1},
    Span, VimwikiIResult, LE,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map, map_parser, not, value, verify},
    multi::{many0, separated_nonempty_list},
    sequence::{delimited, pair, preceded, terminated},
};

#[inline]
pub fn table(input: Span) -> VimwikiIResult<LE<Table>> {
    fn inner(input: Span) -> VimwikiIResult<Table> {
        // Assume a table is centered if the first row is indented
        let (input, (table_header, centered)) =
            map(pair(space0, row), |x| (x.1, !x.0.fragment().is_empty()))(
                input,
            )?;

        // Retrieve remaining rows and prepend the header row
        let (input, mut rows) = many0(preceded(space0, row))(input)?;
        rows.insert(0, table_header);
        Ok((input, Table::new(rows, centered)))
    }

    // Parse the table and make sure it isn't comprised entirely of divider rows
    context(
        "Table",
        le(verify(inner, |t| {
            !t.rows.iter().all(|r| matches!(r.element, Row::Divider))
        })),
    )(input)
}

#[inline]
fn row(input: Span) -> VimwikiIResult<LE<Row>> {
    fn inner(input: Span) -> VimwikiIResult<Row> {
        terminated(
            delimited(
                char('|'),
                alt((
                    map(separated_nonempty_list(char('|'), hyphens), |_| {
                        Row::Divider
                    }),
                    map(separated_nonempty_list(char('|'), cell), Row::from),
                )),
                char('|'),
            ),
            end_of_line_or_input,
        )(input)
    }

    context("Row", le(inner))(input)
}

#[inline]
fn hyphens(input: Span) -> VimwikiIResult<()> {
    value((), take_line_while1(char('-')))(input)
}

#[inline]
fn cell(input: Span) -> VimwikiIResult<LE<Cell>> {
    fn inner(input: Span) -> VimwikiIResult<Cell> {
        alt((
            cell_span_above,
            cell_span_left,
            map(
                map_parser(
                    take_line_while1(not(char('|'))),
                    inline_element_container,
                ),
                |c| c.map(Cell::Content).element,
            ),
        ))(input)
    }

    context("Cell", le(inner))(input)
}

#[inline]
fn cell_span_left(input: Span) -> VimwikiIResult<Cell> {
    value(Cell::SpanLeft, delimited(space0, tag(">"), space0))(input)
}

#[inline]
fn cell_span_above(input: Span) -> VimwikiIResult<Cell> {
    value(Cell::SpanAbove, delimited(space0, tag("\\/"), space0))(input)
}

#[cfg(test)]
mod tests {
    use super::super::elements::{InlineElement, Link, WikiLink};
    use super::*;
    use crate::lang::utils::Span;
    use indoc::indoc;
    use std::path::PathBuf;

    fn check_cell_text_value(cell: &Cell, value: &str) {
        check_cell_value(cell, |c| {
            assert_eq!(c, &InlineElement::Text(value.to_string()));
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
                f(&x.elements[0].element);
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
    fn table_should_succeed_if_uneven_columns_found() {
        let input = Span::from(indoc! {"
        |name| age|
        |----|----|
        |abcd|1111|
        |efgh|2222|3333|
        |ijkl|4444|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "name");

        let cell = &t.get_cell(0, 1).unwrap().element;
        check_cell_text_value(cell, " age");

        assert_eq!(t.rows[1].element, Row::Divider);

        let cell = &t.get_cell(2, 0).unwrap().element;
        check_cell_text_value(cell, "abcd");

        let cell = &t.get_cell(2, 1).unwrap().element;
        check_cell_text_value(cell, "1111");

        let cell = &t.get_cell(3, 0).unwrap().element;
        check_cell_text_value(cell, "efgh");

        let cell = &t.get_cell(3, 1).unwrap().element;
        check_cell_text_value(cell, "2222");

        let cell = &t.get_cell(3, 2).unwrap().element;
        check_cell_text_value(cell, "3333");

        let cell = &t.get_cell(4, 0).unwrap().element;
        check_cell_text_value(cell, "ijkl");

        let cell = &t.get_cell(4, 1).unwrap().element;
        check_cell_text_value(cell, "4444");
    }

    #[test]
    fn table_should_support_single_row_with_single_cell() {
        let input = Span::from("|value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_single_row_with_multiple_cells() {
        let input = Span::from("|value1|value2|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().element;
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_single_cells() {
        let input = Span::from(indoc! {"
        |value1|
        |value2|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(1, 0).unwrap().element;
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_multiple_cells() {
        let input = Span::from(indoc! {"
        |value1|value2|
        |value3|value4|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().element;
        check_cell_text_value(cell, "value2");

        let cell = &t.get_cell(1, 0).unwrap().element;
        check_cell_text_value(cell, "value3");

        let cell = &t.get_cell(1, 1).unwrap().element;
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
            input.fragment().is_empty(),
            "Did not consume table: '{}'",
            input.fragment_str()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");

        assert_eq!(t.rows[1].element, Row::Divider);
    }

    #[test]
    fn table_should_support_row_and_divider_with_multiple_cells() {
        let input = Span::from(indoc! {"
        |value1|value2|
        |------|------|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume table: '{}'",
            input.fragment_str()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().element;
        check_cell_text_value(cell, "value2");

        assert_eq!(t.rows[1].element, Row::Divider);
    }

    #[test]
    fn table_should_support_span_left_cell() {
        let input = Span::from("|>|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        assert_eq!(cell, &Cell::SpanLeft);
    }

    #[test]
    fn table_should_support_span_above_cell() {
        let input = Span::from(r"|\/|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        assert_eq!(cell, &Cell::SpanAbove);
    }

    #[test]
    fn table_should_support_centering_through_indentation() {
        let input = Span::from(" |value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(t.centered, "Table unexpectedly not centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_inline_content_in_cells() {
        let input = Span::from("|[[some link]]|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().element;
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
