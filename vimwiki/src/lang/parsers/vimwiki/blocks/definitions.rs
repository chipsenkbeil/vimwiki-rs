use super::{
    elements::{Definition, DefinitionList, Term, TermAndDefinitions},
    inline::inline_element_container,
    utils::{
        beginning_of_line, context, end_of_line_or_input, le, take_line_while1,
        take_until_end_of_line_or_input, unwrap_le,
    },
    Span, VimwikiIResult, LE,
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, map_parser, not, opt, verify},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
};

#[inline]
pub fn definition_list(input: Span) -> VimwikiIResult<LE<DefinitionList>> {
    context(
        "Definition List",
        le(map(many1(term_and_definitions), DefinitionList::from)),
    )(input)
}

/// Parser that detects a term and one or more definitions
#[inline]
fn term_and_definitions(input: Span) -> VimwikiIResult<TermAndDefinitions> {
    let (input, _) = beginning_of_line(input)?;
    let (input, (term, maybe_def)) = term_line(input)?;
    let (input, mut defs) =
        verify(many0(definition_line), |defs: &Vec<Definition>| {
            maybe_def.is_some() || !defs.is_empty()
        })(input)?;

    if let Some(def) = maybe_def {
        defs.insert(0, def);
    }

    Ok((
        input,
        TermAndDefinitions {
            term,
            definitions: defs,
        },
    ))
}

/// Parsers a line as a term (with optional definition)
#[inline]
fn term_line(input: Span) -> VimwikiIResult<(Term, Option<Definition>)> {
    let (input, _) = beginning_of_line(input)?;

    // Parse our term and provide location information for it
    let (input, term) = terminated(
        map_parser(
            take_line_while1(not(tag("::"))),
            unwrap_le(inline_element_container),
        ),
        tag("::"),
    )(input)?;

    // Now check if we have a definition following
    let (input, maybe_def) = opt(preceded(
        space1,
        map_parser(
            take_until_end_of_line_or_input,
            unwrap_le(inline_element_container),
        ),
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
    let (input, def) = map_parser(
        take_until_end_of_line_or_input,
        unwrap_le(inline_element_container),
    )(input)?;
    let (input, _) = end_of_line_or_input(input)?;

    Ok((input, def))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        elements::{
            DecoratedText, DecoratedTextContent, InlineElement,
            InlineElementContainer, Link, MathInline, Text, WikiLink,
        },
        lang::utils::Span,
    };
    use indoc::indoc;
    use std::path::PathBuf;

    /// Checks defs match those of a provided list in ANY order
    fn check_text_defs(defs: Vec<&Definition>, expected: Vec<&str>) {
        assert_eq!(
            defs.len(),
            expected.len(),
            "Mismatch of expected definitions"
        );
        for d in defs.iter() {
            assert!(
                expected.contains(&d.to_string().as_str()),
                "Definition {} not expected",
                d
            );
        }
    }

    #[test]
    fn definition_list_should_fail_if_input_empty() {
        let input = Span::from("");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_not_starting_with_a_term() {
        let input = Span::from("no term here");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_starting_with_a_definition() {
        let input = Span::from(":: some definition");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_no_space_between_term_and_def_sep() {
        let input = Span::from("term::some definition");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_one_term_and_no_defs() {
        let input = Span::from("term::");
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_fail_if_multiple_terms_and_no_defs() {
        let input = Span::from(indoc! {r#"
            term 1::
            term 2::
        "#});
        assert!(definition_list(input).is_err());
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def() {
        let input = Span::from("term 1:: def 1");
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_def_on_next_line() {
        let input = Span::from(indoc! {r#"
            term 1::
            :: def 1
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_multiple_line_defs() {
        let input = Span::from(indoc! {r#"
            term 1::
            :: def 1
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1", "def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def_and_line_def()
    {
        let input = Span::from(indoc! {r#"
            term 1:: def 1
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1", "def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_inline_defs() {
        let input = Span::from(indoc! {r#"
            term 1:: def 1
            term 2:: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1"]);

        let defs = l.defs_for_term("term 2").unwrap().collect();
        check_text_defs(defs, vec!["def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_line_defs() {
        let input = Span::from(indoc! {r#"
            term 1::
            :: def 1
            term 2::
            :: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1"]);

        let defs = l.defs_for_term("term 2").unwrap().collect();
        check_text_defs(defs, vec!["def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_mixed_defs() {
        let input = Span::from(indoc! {r#"
            term 1:: def 1
            :: def 2
            term 2:: def 3
            :: def 4
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let defs = l.defs_for_term("term 1").unwrap().collect();
        check_text_defs(defs, vec!["def 1", "def 2"]);

        let defs = l.defs_for_term("term 2").unwrap().collect();
        check_text_defs(defs, vec!["def 3", "def 4"]);
    }

    #[test]
    fn definition_list_should_support_inline_elements_in_terms_and_definitions()
    {
        let input = Span::from(indoc! {r#"
            *term* 1:: [[def 1]]
            :: def $2+2$
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume def list");

        let terms: Vec<&Term> = l.terms().collect();
        assert_eq!(
            terms,
            vec![&InlineElementContainer::new(vec![
                LE::from(InlineElement::DecoratedText(DecoratedText::Bold(
                    vec![LE::from(DecoratedTextContent::from(Text::from(
                        "term"
                    )))]
                ))),
                LE::from(InlineElement::Text(Text::from(" 1"))),
            ])]
        );

        let defs: Vec<&Definition> =
            l.defs_for_term("term 1").unwrap().collect();
        assert_eq!(defs.len(), 2, "Wrong number of definitions found");
        assert_eq!(
            defs[0],
            &InlineElementContainer::new(vec![LE::from(InlineElement::from(
                Link::from(WikiLink::new(PathBuf::from("def 1"), None, None))
            ))])
        );
        assert_eq!(
            defs[1],
            &InlineElementContainer::new(vec![
                LE::from(InlineElement::Text(Text::from("def "))),
                LE::from(InlineElement::from(MathInline::new(
                    "2+2".to_string()
                ))),
            ])
        );
    }
}
