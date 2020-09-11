use super::{
    components::{Cell, Row, Table},
    inline_component_container,
    utils::{end_of_line_or_input, lc, position, take_line_while1},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::{map, map_parser, not, value, verify},
    error::context,
    multi::{many0, separated_nonempty_list},
    sequence::{delimited, pair, preceded, terminated},
};

#[inline]
pub fn table(input: Span) -> VimwikiIResult<LC<Table>> {
    fn _table(input: Span) -> VimwikiIResult<Table> {
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
    lc(verify(_table, |t| {
        !t.rows.iter().all(|r| matches!(r.component, Row::Divider))
    }))(input)
}

#[inline]
fn row(input: Span) -> VimwikiIResult<LC<Row>> {
    let (input, pos) = position(input)?;

    let (input, row) = context(
        "Row",
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
        ),
    )(input)?;

    Ok((input, LC::from((row, pos, input))))
}

#[inline]
fn hyphens(input: Span) -> VimwikiIResult<()> {
    value((), take_line_while1(char('-')))(input)
}

#[inline]
fn cell(input: Span) -> VimwikiIResult<LC<Cell>> {
    let (input, pos) = position(input)?;

    let (input, cell) = context(
        "Cell",
        alt((
            cell_span_above,
            cell_span_left,
            map(
                map_parser(
                    take_line_while1(not(char('|'))),
                    inline_component_container,
                ),
                |c| c.map(Cell::Content).component,
            ),
        )),
    )(input)?;

    Ok((input, LC::from((cell, pos, input))))
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
    use super::super::components::{InlineComponent, Link, WikiLink};
    use super::*;
    use indoc::indoc;
    use std::path::PathBuf;

    fn check_cell_text_value(cell: &Cell, value: &str) {
        check_cell_value(cell, |c| {
            assert_eq!(c, &InlineComponent::Text(value.to_string()));
        });
    }

    fn check_cell_value(cell: &Cell, f: impl Fn(&InlineComponent)) {
        match cell {
            Cell::Content(x) => {
                assert_eq!(
                    x.components.len(),
                    1,
                    "Unexpected number of inline components in cell"
                );
                f(&x.components[0].component);
            }
            x => panic!("Unexpected cell: {:?}", x),
        }
    }

    #[test]
    fn table_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_not_starting_with_pipe() {
        let input = Span::new(indoc! {"
        name|age|
        |---|---|
        |abc|012|
        |def|045|
        "});
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_not_ending_with_pipe() {
        let input = Span::new(indoc! {"
        |name|age
        |---|---|
        |abc|012|
        |def|045|
        "});
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_fail_if_no_content_row_found() {
        let input = Span::new("|---------|");
        assert!(table(input).is_err());
    }

    #[test]
    fn table_should_succeed_if_uneven_columns_found() {
        let input = Span::new(indoc! {"
        |name| age|
        |----|----|
        |abcd|1111|
        |efgh|2222|3333|
        |ijkl|4444|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "name");

        let cell = &t.get_cell(0, 1).unwrap().component;
        check_cell_text_value(cell, " age");

        assert_eq!(t.rows[1].component, Row::Divider);

        let cell = &t.get_cell(2, 0).unwrap().component;
        check_cell_text_value(cell, "abcd");

        let cell = &t.get_cell(2, 1).unwrap().component;
        check_cell_text_value(cell, "1111");

        let cell = &t.get_cell(3, 0).unwrap().component;
        check_cell_text_value(cell, "efgh");

        let cell = &t.get_cell(3, 1).unwrap().component;
        check_cell_text_value(cell, "2222");

        let cell = &t.get_cell(3, 2).unwrap().component;
        check_cell_text_value(cell, "3333");

        let cell = &t.get_cell(4, 0).unwrap().component;
        check_cell_text_value(cell, "ijkl");

        let cell = &t.get_cell(4, 1).unwrap().component;
        check_cell_text_value(cell, "4444");
    }

    #[test]
    fn table_should_support_single_row_with_single_cell() {
        let input = Span::new("|value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_single_row_with_multiple_cells() {
        let input = Span::new("|value1|value2|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().component;
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_single_cells() {
        let input = Span::new(indoc! {"
        |value1|
        |value2|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(1, 0).unwrap().component;
        check_cell_text_value(cell, "value2");
    }

    #[test]
    fn table_should_support_multiple_rows_with_multiple_cells() {
        let input = Span::new(indoc! {"
        |value1|value2|
        |value3|value4|
        "});
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().component;
        check_cell_text_value(cell, "value2");

        let cell = &t.get_cell(1, 0).unwrap().component;
        check_cell_text_value(cell, "value3");

        let cell = &t.get_cell(1, 1).unwrap().component;
        check_cell_text_value(cell, "value4");
    }

    #[test]
    fn table_should_support_row_and_divider_with_single_cell() {
        let input = Span::new(indoc! {"
        |value1|
        |------|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume table: '{}'",
            input.fragment()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");

        assert_eq!(t.rows[1].component, Row::Divider);
    }

    #[test]
    fn table_should_support_row_and_divider_with_multiple_cells() {
        let input = Span::new(indoc! {"
        |value1|value2|
        |------|------|
        "});
        let (input, t) = table(input).unwrap();
        assert!(
            input.fragment().is_empty(),
            "Did not consume table: '{}'",
            input.fragment()
        );
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");

        let cell = &t.get_cell(0, 1).unwrap().component;
        check_cell_text_value(cell, "value2");

        assert_eq!(t.rows[1].component, Row::Divider);
    }

    #[test]
    fn table_should_support_span_left_cell() {
        let input = Span::new("|>|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        assert_eq!(cell, &Cell::SpanLeft);
    }

    #[test]
    fn table_should_support_span_above_cell() {
        let input = Span::new(r"|\/|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        assert_eq!(cell, &Cell::SpanAbove);
    }

    #[test]
    fn table_should_support_centering_through_indentation() {
        let input = Span::new(" |value1|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(t.centered, "Table unexpectedly not centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_text_value(cell, "value1");
    }

    #[test]
    fn table_should_support_inline_content_in_cells() {
        let input = Span::new("|[[some link]]|");
        let (input, t) = table(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume table");
        assert!(!t.centered, "Table unexpectedly centered");

        let cell = &t.get_cell(0, 0).unwrap().component;
        check_cell_value(cell, |c| {
            assert_eq!(
                c,
                &InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("some link")
                )))
            );
        });
    }
}
