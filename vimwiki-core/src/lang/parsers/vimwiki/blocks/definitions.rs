use crate::lang::{
    elements::{
        Definition, DefinitionList, InlineElementContainer, Located, Term,
    },
    parsers::{
        utils::{
            beginning_of_line, capture, context, deeper, end_of_line_or_input,
            locate, take_line_until1, take_until_end_of_line_or_input,
        },
        vimwiki::blocks::inline::inline_element_container,
        IResult, Span,
    },
};
use nom::{
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{map, map_parser, opt, verify},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
};
use std::iter::FromIterator;

#[inline]
pub fn definition_list(input: Span) -> IResult<Located<DefinitionList>> {
    context(
        "Definition List",
        locate(capture(map(
            many1(deeper(term_and_definitions)),
            DefinitionList::from_iter,
        ))),
    )(input)
}

/// Parser that detects a term and one or more definitions
#[inline]
fn term_and_definitions<'a>(
    input: Span<'a>,
) -> IResult<(Located<Term<'a>>, Vec<Located<Definition<'a>>>)> {
    let (input, _) = beginning_of_line(input)?;
    let (input, (term, maybe_def)) = term_line(input)?;
    let (input, mut defs) =
        verify(many0(definition_line), |defs: &Vec<Located<Definition>>| {
            maybe_def.is_some() || !defs.is_empty()
        })(input)?;

    if let Some(def) = maybe_def {
        defs.insert(0, def);
    }

    Ok((input, (term, defs)))
}

/// Parsers a line as a term (with optional definition)
#[inline]
fn term_line(
    input: Span,
) -> IResult<(Located<Term>, Option<Located<Definition>>)> {
    let (input, _) = beginning_of_line(input)?;

    // Parse our term and provide location information for it
    let (input, term) = locate(capture(terminated(
        map_parser(
            take_line_until1("::"),
            map(
                inline_element_container,
                |l: Located<InlineElementContainer>| Term::new(l.into_inner()),
            ),
        ),
        tag("::"),
    )))(input)?;

    // Now check if we have a definition following
    let (input, maybe_def) = opt(locate(capture(preceded(
        space1,
        map_parser(
            take_until_end_of_line_or_input,
            map(
                inline_element_container,
                |l: Located<InlineElementContainer>| {
                    Definition::new(l.into_inner())
                },
            ),
        ),
    ))))(input)?;

    // Conclude with any lingering space and newline
    let (input, _) = pair(space0, end_of_line_or_input)(input)?;

    Ok((input, (term, maybe_def)))
}

/// Parses a line as a definition
#[inline]
fn definition_line(input: Span) -> IResult<Located<Definition>> {
    fn inner(input: Span) -> IResult<Definition> {
        let (input, _) = beginning_of_line(input)?;
        let (input, _) = tag("::")(input)?;
        let (input, _) = space1(input)?;
        let (input, def) = map_parser(
            take_until_end_of_line_or_input,
            map(
                inline_element_container,
                |l: Located<InlineElementContainer>| l.into_inner(),
            ),
        )(input)?;
        let (input, _) = end_of_line_or_input(input)?;

        Ok((input, Definition::new(def)))
    }

    context("Definition Line", locate(capture(inner)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::{
        DecoratedText, DecoratedTextContent, InlineElement,
        InlineElementContainer, Link, MathInline, Text,
    };
    use indoc::indoc;
    use std::convert::TryFrom;
    use uriparse::URIReference;

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
    fn definition_list_should_properly_adjust_depth_for_children() {
        let input = Span::from("term1:: def1");
        let (_, def_list) = definition_list(input).unwrap();
        assert_eq!(
            def_list.depth(),
            0,
            "Definition list depth was at wrong level"
        );
        for term in def_list.terms() {
            assert_eq!(term.depth(), 1, "Term depth was at wrong level");
        }
        for def in def_list.definitions() {
            assert_eq!(def.depth(), 1, "Definition depth was at wrong level");
        }
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_inline_def() {
        let input = Span::from("term 1:: def 1");
        let (input, l) = definition_list(input).unwrap();
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        check_text_defs(defs, vec!["def 1"]);
    }

    #[test]
    fn definition_list_should_succeed_if_one_term_and_def_on_next_line() {
        let input = Span::from(indoc! {r#"
            term 1::
            :: def 1
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
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
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
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
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        check_text_defs(defs, vec!["def 1", "def 2"]);
    }

    #[test]
    fn definition_list_should_succeed_if_multiple_terms_and_inline_defs() {
        let input = Span::from(indoc! {r#"
            term 1:: def 1
            term 2:: def 2
        "#});
        let (input, l) = definition_list(input).unwrap();
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        check_text_defs(defs, vec!["def 1"]);

        let defs = l
            .get("term 2")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
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
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        check_text_defs(defs, vec!["def 1"]);

        let defs = l
            .get("term 2")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
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
        assert!(input.is_empty(), "Did not consume def list");

        let defs = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        check_text_defs(defs, vec!["def 1", "def 2"]);

        let defs = l
            .get("term 2")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
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
        assert!(input.is_empty(), "Did not consume def list");

        let terms: Vec<&Term> = l.terms().map(Located::as_inner).collect();
        assert_eq!(
            terms,
            vec![&InlineElementContainer::new(vec![
                Located::from(InlineElement::DecoratedText(
                    DecoratedText::Bold(vec![Located::from(
                        DecoratedTextContent::from(Text::from("term"))
                    )])
                )),
                Located::from(InlineElement::Text(Text::from(" 1"))),
            ])]
        );

        let defs: Vec<&Definition> = l
            .get("term 1")
            .unwrap()
            .iter()
            .map(Located::as_inner)
            .collect();
        assert_eq!(defs.len(), 2, "Wrong number of definitions found");
        assert_eq!(
            defs[0],
            &InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Link::new_wiki_link(
                    URIReference::try_from("def%201").unwrap(),
                    None
                ))
            )])
        );
        assert_eq!(
            defs[1],
            &InlineElementContainer::new(vec![
                Located::from(InlineElement::Text(Text::from("def "))),
                Located::from(InlineElement::from(MathInline::from("2+2"))),
            ])
        );
    }
}
