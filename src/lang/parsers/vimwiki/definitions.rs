use super::{
    components::{Definition, DefinitionList, Term},
    utils::{
        beginning_of_line, end_of_line_or_input, lc, position, pstring,
        take_line_while1, take_until_end_of_line_or_input,
    },
    Span, VimwikiIResult, LC,
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, not, opt, verify},
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
    let (input, mut defs) =
        verify(many0(definition_line), |defs: &Vec<Definition>| {
            maybe_def.is_some() || !defs.is_empty()
        })(input)?;

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
    let (input, maybe_def) = opt(preceded(
        space1,
        lc(pstring(take_until_end_of_line_or_input)),
    ))(input)?;

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
    let (input, def) = lc(pstring(take_until_end_of_line_or_input))(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, def))
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    /// Checks defs match those of a provided list in ANY order
    fn check_text_defs(defs: Vec<&Definition>, expected: Vec<&str>) {
        assert_eq!(
            defs.len(),
            expected.len(),
            "Mismatch of expected definitions"
        );
        for d in defs.iter() {
            assert!(
                expected.contains(&d.component.as_str()),
                "Definition {} not expected",
                d.component
            );
        }
    }

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
    fn definition_list_should_fail_if_one_term_and_no_defs() {
        let input = Span::new("term::");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_multiple_terms_and_no_defs() {
        let input = Span::new(indoc! {r#"
            term 1::
            term 2::
        "#});
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def() {
        let input = Span::new("term 1:: def 1");
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_def_on_next_line() {
        let input = Span::new(indoc! {r#"
            term 1::
            :: def 1
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_multiple_line_defs() {
        let input = Span::new(indoc! {r#"
            term 1::
            :: def 1
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1", "def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def_and_line_def()
    {
        let input = Span::new(indoc! {r#"
            term 1:: def 1
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1", "def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_inline_defs() {
        let input = Span::new(indoc! {r#"
            term 1:: def 1
            term 2:: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1"]);

        let defs = l.find_definitions(&"term 2".into());
        check_text_defs(defs, vec!["def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_line_defs() {
        let input = Span::new(indoc! {r#"
            term 1::
            :: def 1
            term 2::
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1"]);

        let defs = l.find_definitions(&"term 2".into());
        check_text_defs(defs, vec!["def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_mixed_defs() {
        let input = Span::new(indoc! {r#"
            term 1:: def 1
            :: def 2
            term 2:: def 3
            :: def 4
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");
        println!("{:?}", l);

        let defs = l.find_definitions(&"term 1".into());
        check_text_defs(defs, vec!["def 1", "def 2"]);

        let defs = l.find_definitions(&"term 2".into());
        check_text_defs(defs, vec!["def 3", "def 4"]);
    }
}
