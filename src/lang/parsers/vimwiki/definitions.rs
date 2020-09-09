use super::{
    components::{Definition, DefinitionList, Term},
    inline_component_container,
    utils::{
        beginning_of_line, end_of_line_or_input, position, take_line_while1,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, not, opt},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
};

#[inline]
pub fn definition_list(input: Span) -> VimwikiIResult<LC<DefinitionList>> {
    let mut dl = DefinitionList::new();

    let (input, pos) = position(input)?;
    let (input, mut terms_and_definitions) =
        many1(term_and_definitions)(input)?;

    // Build our term and definition list based on parsed information
    for (term, mut defs) in terms_and_definitions.drain(..) {
        let tid = dl.add_term(term);
        for def in defs.drain(..) {
            let did = dl.add_definition(def);
            dl.connect_term_to_definition(tid, did);
        }
    }

    Ok((input, LC::from((dl, pos, input))))
}

/// Parser that detects a term and one or more definitions
#[inline]
fn term_and_definitions(
    input: Span,
) -> VimwikiIResult<(Term, Vec<Definition>)> {
    let (input, _) = beginning_of_line(input)?;
    let (input, (term, maybe_def)) = term_line(input)?;
    let (input, mut defs) = many0(definition_line)(input)?;

    if let Some(def) = maybe_def {
        defs.insert(0, def);
    }

    Ok((input, (term, defs)))
}

/// Parsers a line as a term (with optional definition)
#[inline]
fn term_line(input: Span) -> VimwikiIResult<(Term, Option<Definition>)> {
    let (input, _) = beginning_of_line(input)?;
    let (input, pos) = position(input)?;

    // Parse our term and provide location information for it
    let (input, term) = terminated(
        map(take_line_while1(not(tag("::"))), |s: Span| {
            LC::from((s.fragment().to_string(), pos, input))
        }),
        tag("::"),
    )(input)?;

    // Now check if we have a definition following
    let (input, maybe_def) =
        opt(preceded(space1, inline_component_container))(input)?;

    // Conclude with any lingering space and newline
    let (input, _) = pair(space0, end_of_line_or_input)(input)?;

    Ok((input, (term, maybe_def)))
}

/// Parses a line as a definition
#[inline]
fn definition_line(input: Span) -> VimwikiIResult<Definition> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("::")(input)?;
    let (input, _) = space1(input)?;
    let (input, def) = inline_component_container(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, def))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_list_should_fail_if_input_empty() {
        let input = Span::new("");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_not_starting_with_a_term() {
        let input = Span::new("no term here");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_starting_with_a_definition() {
        let input = Span::new(":: some definition");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_no_space_between_term_and_def_sep() {
        let input = Span::new("term::some definition");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_no_defs() {
        let input = Span::new("term::");
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_def_on_next_line() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_multiple_line_defs() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def_and_line_def()
    {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_no_defs() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_inline_defs() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_line_defs() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_mixed_defs() {
        todo!();
    }
}
