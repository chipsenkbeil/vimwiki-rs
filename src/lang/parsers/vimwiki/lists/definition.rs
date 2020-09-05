use super::{
    components::{Definition, DefinitionList, Term},
    inline_component_container,
    utils::{beginning_of_line, end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{anychar, space1},
    combinator::{map, opt, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded},
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

    // Parse our definition and provide location information for it
    let (input, term) = map(
        recognize(many1(preceded(
            pair(tag("::"), end_of_line_or_input),
            anychar,
        ))),
        |s: Span| LC::from((s.fragment().to_string(), pos, input)),
    )(input)?;

    // Now check if we have a definition following
    let (input, maybe_def) = opt(delimited(
        space1,
        inline_component_container,
        end_of_line_or_input,
    ))(input)?;

    Ok((input, (term, maybe_def.map(|c| c.component))))
}

/// Parses a line as a definition
#[inline]
fn definition_line(input: Span) -> VimwikiIResult<Definition> {
    let (input, _) = beginning_of_line(input)?;
    let (input, _) = tag("::")(input)?;
    let (input, _) = space1(input)?;
    let (input, def) = inline_component_container(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, def.component))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_list_should_fail_if_not_starting_with_a_term() {
        todo!();
    }

    #[test]
    fn definition_list_should_fail_if_no_space_between_term_and_def_sep() {
        todo!();
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_no_defs() {
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
