use super::{
    components::{Cell, Row, Table},
    inline_component_container,
    utils::{end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, space0},
    combinator::{map, map_parser, value},
    error::context,
    multi::{many0, separated_nonempty_list},
    sequence::{delimited, pair, preceded, terminated},
};

#[inline]
pub fn table(input: Span) -> VimwikiIResult<LC<Table>> {
    let (input, pos) = position(input)?;

    // Assume a table is centered if the first row is indented
    let (input, (table_header, centered)) =
        map(pair(space0, row), |x| (x.1, !x.0.fragment().is_empty()))(input)?;

    // Retrieve remaining rows and prepend the header row
    let (input, mut rows) = many0(preceded(space0, row))(input)?;
    rows.insert(0, table_header);

    Ok((input, LC::from((Table::new(rows, centered), pos, input))))
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
    value((), take_while1(|c| c == '-'))(input)
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
                    take_while1(|c| c != '|'),
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
    use super::*;

    #[test]
    fn table_should_fail_if_not_starting_with_pipe() {
        todo!();
    }

    #[test]
    fn table_should_fail_if_not_ending_with_pipe() {
        todo!();
    }

    #[test]
    fn table_should_fail_if_no_content_row_found() {
        todo!();
    }

    #[test]
    fn table_should_support_single_row_with_single_cell() {
        todo!();
    }

    #[test]
    fn table_should_support_single_row_with_multiple_cells() {
        todo!();
    }

    #[test]
    fn table_should_support_multiple_rows_with_single_cells() {
        todo!();
    }

    #[test]
    fn table_should_support_multiple_rows_with_multiple_cells() {
        todo!();
    }
}
