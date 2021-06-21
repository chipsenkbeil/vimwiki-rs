use crate::lang::{
    elements::{
        BlockElement, List, ListItem, ListItemAttributes, ListItemContents,
        ListItemSuffix, ListItemTodoStatus, ListItemType, Located,
        OrderedListItemType, UnorderedListItemType,
    },
    parsers::{
        utils::{
            beginning_of_line, capture, context, deeper, locate, rest_of_line,
        },
        vimwiki::blocks::nested_block_element,
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, one_of, space0},
    combinator::{map, opt, peek, recognize, value, verify},
    multi::{fold_many0, many0, many1},
    sequence::{pair, preceded},
};

#[inline]
pub fn list(input: Span) -> IResult<Located<List>> {
    fn inner(input: Span) -> IResult<List> {
        // A list must at least have one item, whose indentation level we will
        // use to determine how far to go
        let (input, (indentation, item)) = deeper(list_item)(input)?;

        // NOTE: Keep track of indentation level for a list based on its first
        //       item
        //
        //       1. Any item with a greater indentation level is a sublist
        //       2. Any item with equal indentation level MUST have matching
        //          prefix type
        //       3. Continuing a list's content must match indentation level to
        //          point where list content starts (not prefix) and must not
        //          start with any other list prefix
        //       4. Any item with less indentation terminates a list
        //       5. Non-blank line not starting with a list item terminates a list
        //       6. Blank line terminates a list
        let (input, (_, items)) = fold_many0(
            preceded(
                verify(indentation_level(false), |level| *level == indentation),
                map(deeper(list_item), |x| x.1),
            ),
            (1, vec![item]),
            |(index, mut items), mut item| {
                // NOTE: The index information isn't available to the list_item
                //       parser, so we have to assign it here
                item.pos = index;

                items.push(item);
                (index + 1, items)
            },
        )(input)?;

        // NOTE: When parsing list item types, we aren't able to distinguish
        //       alphabetic versus roman numerals as they both involve
        //       alphabetic characters. We need to analyze the entire list after
        //       it is created to see if all items resolved to roman numerals,
        //       otherwise we will need to convert types to alphabetic instead
        Ok((input, List::new(items).normalize().to_owned()))
    }

    context("List", locate(capture(inner)))(input)
}

/// Parse space/tabs before a list item, followed by the list item
#[inline]
pub fn list_item(input: Span) -> IResult<(usize, Located<ListItem>)> {
    fn inner(input: Span) -> IResult<(usize, Located<ListItem>)> {
        // 1. Start at the beginning of the line
        let (input, _) = beginning_of_line(input)?;

        // 2. Determine the indentation level of this list item
        let (input, indentation) = indentation_level(true)(input)?;

        // 3. Grab input up to the next list item or other item based on the
        //    indentation level
        let (_, remaining) = recognize(pair(
            rest_of_line,
            many0(preceded(
                verify(indentation_level(false), |level| *level > indentation),
                rest_of_line,
            )),
        ))(input)?;

        // 4. Ensure that the item starts with a valid prefix
        let (remaining, item) = locate(capture(map(
            pair(list_item_prefix, list_item_tail(indentation)),
            |((item_type, item_suffix), (attrs, contents))| {
                // NOTE: To make things easier, we aren't assigning the index
                //       within this parser; rather, we put a filler index and
                //       will assign the actual index in the parent parser
                ListItem::new(item_type, item_suffix, 0, contents, attrs)
            },
        )))(remaining)?;

        // 5. Add back in all remaining that was not consumed as it is not
        //    part of the list item; this happens if a header is following
        //    immediately as it is not part of a nested block element but
        //    can still be successfully parsed because of its support of
        //    0+ spaces as part of centering
        let input = input
            .advance_start_by(remaining.start_offset() - input.start_offset());

        Ok((input, (indentation, item)))
    }

    context("List Item", inner)(input)
}

#[inline]
fn list_item_tail(
    indentation: usize,
) -> impl Fn(Span) -> IResult<(ListItemAttributes, ListItemContents)> {
    move |input: Span| {
        // 4. Check if we have a todo status attribute
        let (input, maybe_todo_status) = opt(todo_status)(input)?;

        // 5. Parse the rest of the current line
        let (input, content) =
            map(deeper(nested_block_element), |c| c.map(BlockElement::from))(
                input,
            )?;

        // 6. Continue parsing additional lines as content for the
        //    current list item as long as the following are met:
        //
        //    a. The indentation must be GREATER than that of the current item,
        //       otherwise the line would either be a sibling line item or
        //       a parent line item
        //    b. The line is not blank
        //
        //    Note that each following line can be additional content or the
        //    start of a sublist, so we need to check for each
        let (input, mut contents) = many0(preceded(
            verify(indentation_level(false), |level| *level > indentation),
            map(deeper(nested_block_element), |c| c.map(BlockElement::from)),
        ))(input)?;

        contents.insert(0, content);

        Ok((
            input,
            (
                ListItemAttributes {
                    todo_status: maybe_todo_status,
                },
                contents.into_iter().collect(),
            ),
        ))
    }
}

/// Parser that determines the indentation level of the current line based
/// on its current position
#[inline]
fn indentation_level(consume: bool) -> impl Fn(Span) -> IResult<usize> {
    move |input: Span| {
        if consume {
            map(space0, |s: Span| s.remaining_len())(input)
        } else {
            map(peek(space0), |s: Span| s.remaining_len())(input)
        }
    }
}

#[inline]
fn todo_status(input: Span) -> IResult<ListItemTodoStatus> {
    let (input, _) = tag("[")(input)?;
    let (input, attr) = alt((
        value(ListItemTodoStatus::Incomplete, tag(" ")),
        value(ListItemTodoStatus::PartiallyComplete1, tag(".")),
        value(ListItemTodoStatus::PartiallyComplete2, tag("o")),
        value(ListItemTodoStatus::PartiallyComplete3, tag("O")),
        value(ListItemTodoStatus::Complete, tag("X")),
        value(ListItemTodoStatus::Rejected, tag("-")),
    ))(input)?;
    let (input, _) = tag("] ")(input)?;
    Ok((input, attr))
}

#[inline]
fn list_item_prefix(input: Span) -> IResult<(ListItemType, ListItemSuffix)> {
    alt((
        map(unordered_list_item_prefix, |(t, s)| {
            (ListItemType::from(t), s)
        }),
        map(ordered_list_item_prefix, |(t, s)| {
            (ListItemType::from(t), s)
        }),
    ))(input)
}

/// Parses the prefix, including the tailing required space, of an unordered
/// list item
///
/// ### Example
///
/// - Some list item
/// * Some other list item
///
#[inline]
fn unordered_list_item_prefix(
    input: Span,
) -> IResult<(UnorderedListItemType, ListItemSuffix)> {
    let (input, item_type) = alt((
        unordered_list_item_type_hyphen,
        unordered_list_item_type_asterisk,
    ))(input)?;

    Ok((input, (item_type, ListItemSuffix::default())))
}

/// Parses the prefix, including the tailing required space, of an ordered
/// list item
///
/// ### Example
///
/// 1. Some list item
/// 1) Some other list item
///
/// aaa. Some other list item
/// AAA. Some other list item
/// aaa) Some other list item
/// AAA) Some other list item
///
/// iii. Some other list item
/// III. Some other list item
/// iii) Some other list item
/// III) Some other list item
///
/// # Some other list item
///
#[inline]
fn ordered_list_item_prefix(
    input: Span,
) -> IResult<(OrderedListItemType, ListItemSuffix)> {
    // NOTE: Roman numeral check comes before alphabetic as alphabetic would
    //       also match roman numerals
    let (input, (item_type, item_suffix)) = alt((
        pair(
            alt((
                ordered_list_item_type_number,
                ordered_list_item_type_lower_roman,
                ordered_list_item_type_upper_roman,
                ordered_list_item_type_lower_alphabet,
                ordered_list_item_type_upper_alphabet,
            )),
            alt((list_item_suffix_period, list_item_suffix_paren)),
        ),
        pair(ordered_list_item_type_pound, list_item_suffix_none),
    ))(input)?;

    Ok((input, (item_type, item_suffix)))
}

#[inline]
fn unordered_list_item_type_hyphen(
    input: Span,
) -> IResult<UnorderedListItemType> {
    value(UnorderedListItemType::Hyphen, tag("- "))(input)
}

#[inline]
fn unordered_list_item_type_asterisk(
    input: Span,
) -> IResult<UnorderedListItemType> {
    value(UnorderedListItemType::Asterisk, tag("* "))(input)
}

#[inline]
fn ordered_list_item_type_number(input: Span) -> IResult<OrderedListItemType> {
    value(OrderedListItemType::Number, digit1)(input)
}

#[inline]
fn ordered_list_item_type_pound(input: Span) -> IResult<OrderedListItemType> {
    value(OrderedListItemType::Pound, tag("#"))(input)
}

#[inline]
fn ordered_list_item_type_lower_alphabet(
    input: Span,
) -> IResult<OrderedListItemType> {
    value(
        OrderedListItemType::LowercaseAlphabet,
        take_while1(|b: u8| {
            let c = char::from(b);
            c.is_ascii_alphabetic() && c.is_ascii_lowercase()
        }),
    )(input)
}

#[inline]
fn ordered_list_item_type_upper_alphabet(
    input: Span,
) -> IResult<OrderedListItemType> {
    value(
        OrderedListItemType::UppercaseAlphabet,
        take_while1(|b: u8| {
            let c = char::from(b);
            c.is_ascii_alphabetic() && c.is_ascii_uppercase()
        }),
    )(input)
}

#[inline]
fn ordered_list_item_type_lower_roman(
    input: Span,
) -> IResult<OrderedListItemType> {
    value(
        OrderedListItemType::LowercaseRoman,
        many1(one_of("ivxlcdm")),
    )(input)
}

#[inline]
fn ordered_list_item_type_upper_roman(
    input: Span,
) -> IResult<OrderedListItemType> {
    value(
        OrderedListItemType::UppercaseRoman,
        many1(one_of("IVXLEDM")),
    )(input)
}

#[inline]
fn list_item_suffix_paren(input: Span) -> IResult<ListItemSuffix> {
    value(ListItemSuffix::Paren, tag(") "))(input)
}

#[inline]
fn list_item_suffix_period(input: Span) -> IResult<ListItemSuffix> {
    value(ListItemSuffix::Period, tag(". "))(input)
}

#[inline]
fn list_item_suffix_none(input: Span) -> IResult<ListItemSuffix> {
    value(ListItemSuffix::None, tag(" "))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::elements::*;
    use indoc::indoc;
    use std::convert::TryFrom;
    use uriparse::URIReference;

    fn check_single_line_list_item(
        l: &List,
        item_type: ListItemType,
        item_suffix: ListItemSuffix,
        text: &str,
    ) {
        let item = &l[0].as_inner();
        assert_eq!(item.ty, item_type);
        assert_eq!(item.suffix, item_suffix);
        assert_eq!(item.pos, 0);

        let actual = match &item[0].as_inner() {
            BlockElement::Paragraph(x) => x[0].to_string(),
            x => panic!("Unexpected item content: {:?}", x),
        };
        assert_eq!(actual, text);
    }

    #[test]
    fn list_should_fail_if_no_proper_start_to_single_list_item() {
        let input = Span::from("| some item with bad prefix");
        assert!(list(input).is_err());
    }

    #[test]
    fn list_should_fail_if_no_space_after_single_list_item_prefix() {
        assert!(list(Span::from("-some item with no space")).is_err());
        assert!(list(Span::from("*some item with no space")).is_err());
        assert!(list(Span::from("1.some item with no space")).is_err());
        assert!(list(Span::from("1)some item with no space")).is_err());
        assert!(list(Span::from("a)some item with no space")).is_err());
        assert!(list(Span::from("A)some item with no space")).is_err());
        assert!(list(Span::from("i)some item with no space")).is_err());
        assert!(list(Span::from("I)some item with no space")).is_err());
        assert!(list(Span::from("#some item with no space")).is_err());
    }

    #[test]
    fn list_should_properly_adjust_depth_for_list_items_and_contents() {
        let input = Span::from(indoc! {"
            - list item 1
              has extra content
              - sublist item 1
                has content
              - sublist item 2
              on multiple lines
            - list item 2
        "});

        let (_, lst) = list(input).unwrap();
        assert_eq!(lst.depth(), 0, "List depth was at wrong level");
        for item in lst.iter() {
            assert_eq!(item.depth(), 1, "List item depth was at wrong level");
            for content in item.contents.iter() {
                assert_eq!(
                    content.depth(),
                    2,
                    "List item inner content was at wrong level"
                );
            }
        }
    }

    #[test]
    fn list_should_succeed_for_single_unordered_hyphen_item() {
        let input = Span::from("- list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_unordered_asterisk_item() {
        let input = Span::from("* list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(UnorderedListItemType::Asterisk),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_pound_item() {
        let input = Span::from("# list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::Pound),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_number_period_item() {
        let input = Span::from("1. list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::Number),
            ListItemSuffix::Period,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_number_paren_item() {
        let input = Span::from("1) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::Number),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_lowercase_alphabet_paren_item() {
        let input = Span::from("a) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::LowercaseAlphabet),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_uppercase_alphabet_paren_item() {
        let input = Span::from("A) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::UppercaseAlphabet),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_lowercase_roman_paren_item() {
        let input = Span::from("i) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::LowercaseRoman),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_uppercase_roman_paren_item() {
        let input = Span::from("I) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::UppercaseRoman),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_support_list_item_with_paragraph_on_same_line() {
        let input = Span::from(indoc! {r#"
            - list *item 1* has a [[link]] with :tag: and $formula$ is DONE
        "#});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list item");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l[0][0].as_paragraph().unwrap(),
            &Paragraph::new(vec![InlineElementContainer::new(vec![
                Located::from(InlineElement::Text(Text::from("list "))),
                Located::from(InlineElement::DecoratedText(
                    DecoratedText::Bold(vec![Located::from(
                        DecoratedTextContent::from(Text::from("item 1"))
                    )])
                )),
                Located::from(InlineElement::Text(Text::from(" has a "))),
                Located::from(InlineElement::Link(Link::new_wiki_link(
                    URIReference::try_from("link").unwrap(),
                    None
                ))),
                Located::from(InlineElement::Text(Text::from(" with "))),
                Located::from(InlineElement::Tags(Tags::from("tag"))),
                Located::from(InlineElement::Text(Text::from(" and "))),
                Located::from(InlineElement::Math(MathInline::from("formula"))),
                Located::from(InlineElement::Text(Text::from(" is "))),
                Located::from(InlineElement::Keyword(Keyword::Done)),
            ])]),
        );
    }

    #[test]
    fn list_should_support_list_item_with_paragraph_on_multiple_lines() {
        let input = Span::from(indoc! {"
            - list item 1
              has extra content
              on multiple lines
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "not a list item\n",
            "Unexpectedly consumed another element"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l[0][0].as_paragraph().unwrap(),
            &Paragraph::new(vec![
                InlineElementContainer::new(vec![Located::from(
                    InlineElement::Text(Text::from("list item 1"))
                ),]),
                InlineElementContainer::new(vec![Located::from(
                    InlineElement::Text(Text::from("has extra content"))
                ),]),
                InlineElementContainer::new(vec![Located::from(
                    InlineElement::Text(Text::from("on multiple lines"))
                ),])
            ]),
        );
    }

    #[test]
    fn list_should_support_list_item_with_blockquote() {
        let input = Span::from(indoc! {"
            - list item
              > some blockquote
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly did not consume input");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
        assert_eq!(
            l[0][1].as_blockquote().unwrap(),
            &vec!["some blockquote"].into_iter().collect::<Blockquote>(),
        );
    }

    #[test]
    fn list_should_support_list_item_with_code_block() {
        let input = Span::from(indoc! {"
            - list item
              {{{
              some code
              }}}
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly did not consume input");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
        assert_eq!(
            l[0][1].as_code_block().unwrap(),
            &CodeBlock::from_lines(vec!["some code"]),
        );
    }

    #[test]
    fn list_should_support_list_item_with_definition_list() {
        let input = Span::from(indoc! {"
            - list item
              term:: def
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly did not consume input");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
        assert_eq!(
            l[0][1].as_definition_list().unwrap(),
            &vec![(
                Located::from(Term::from("term")),
                vec![Located::from(Definition::from("def"))]
            )]
            .into_iter()
            .collect::<DefinitionList>(),
        );
    }

    #[test]
    fn list_should_support_list_item_with_math_block() {
        let input = Span::from(indoc! {"
            - list item
              {{$
              some math
              }}$
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly did not consume input");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
        assert_eq!(
            l[0][1].as_math_block().unwrap(),
            &MathBlock::from_lines(vec!["some math"]),
        );
    }

    #[test]
    fn list_should_support_list_item_with_table() {
        let input = Span::from(indoc! {"
            - list item
              |cell|
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Unexpectedly did not consume input");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
        assert_eq!(
            l[0][1].as_table().unwrap(),
            &Table::new(
                vec![(
                    CellPos { row: 0, col: 0 },
                    Located::from(Cell::Content(InlineElementContainer::new(
                        vec![Located::from(InlineElement::Text(Text::from(
                            "cell"
                        )))]
                    )))
                )],
                false
            ),
        );
    }

    #[test]
    fn list_should_not_support_list_item_with_divider() {
        let input = Span::from(indoc! {"
            - list item
              ----
        "});
        let (input, l) = list(input).unwrap();
        assert!(
            input.is_empty(),
            "Did not consume divider as part of paragraph"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l[0][0].as_paragraph().unwrap().to_string(),
            "list item\n----"
        );
    }

    #[test]
    fn list_should_support_list_item_with_header() {
        let input = Span::from(indoc! {"
            - list item
              =header=
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "  =header=\n",
            "Unexpectedly consumed header"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item");
    }

    #[test]
    fn list_should_not_support_list_item_with_placeholder() {
        let input = Span::from(indoc! {"
            - list item
              %title my title
        "});
        let (input, l) = list(input).unwrap();
        assert!(
            input.is_empty(),
            "Did not consume placeholder as part of paragraph"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l[0][0].as_paragraph().unwrap().to_string(),
            "list item\n%title my title"
        );
    }

    #[test]
    fn list_should_support_list_item_with_sublists() {
        let input = Span::from(indoc! {"
            - list item 1
              - sublist item 1
              - sublist item 2
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "not a list item\n",
            "Unexpectedly consumed another element"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        // First, have a paragraph on the first line
        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item 1",);

        // Second, have a sublist with two items
        let sublist = l[0][1].as_list().unwrap();
        assert_eq!(sublist.len(), 2, "Unexpected number of list items");
        assert_eq!(
            sublist[0][0].as_paragraph().unwrap().to_string(),
            "sublist item 1",
        );
        assert_eq!(
            sublist[1][0].as_paragraph().unwrap().to_string(),
            "sublist item 2",
        );
    }

    #[test]
    fn list_should_support_list_item_with_content_separated_by_sublist() {
        let input = Span::from(indoc! {"
            - list item 1
              has extra content
              - sublist item 1
                has content
              - sublist item 2
              on multiple lines
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(
            input.as_unsafe_remaining_str(),
            "not a list item\n",
            "Unexpectedly consumed another element"
        );
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        // First, have a paragraph on the first two lines
        assert_eq!(
            l[0][0].as_paragraph().unwrap().to_string(),
            "list item 1\nhas extra content",
        );

        // Second, have a sublist
        let sublist = l[0][1].as_list().unwrap();
        assert_eq!(sublist.len(), 2, "Unexpected number of list items");

        assert_eq!(
            sublist[0][0].as_paragraph().unwrap().to_string(),
            "sublist item 1\nhas content",
        );

        assert_eq!(
            sublist[1][0].as_paragraph().unwrap().to_string(),
            "sublist item 2",
        );

        // Third, have another paragraph on last line
        assert_eq!(
            l[0][2].as_paragraph().unwrap().to_string(),
            "on multiple lines",
        );
    }

    #[test]
    fn list_should_supported_deeply_nested_sublists_with_paragraphs_over_blockquotes(
    ) {
        // NOTE: This comes from a specific case I encountered where deeply
        //       nesting list item content with paragraphs is still getting
        //       parsed as blockquotes such as "and content after that sublist"
        //       because it is not already part of a paragraph
        let input = Span::from(indoc! {"
            - List of items
                - Containing a sublist
                    - With another sublist
                        - And an additional sublist
                      with content from the a sublist
                  and content after that sublist as paragraph within list item
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l[0][0].as_paragraph().unwrap().to_string(),
            "List of items"
        );

        let sublist = l[0][1].as_list().unwrap();
        assert_eq!(
            sublist[0][0].as_paragraph().unwrap().to_string(),
            "Containing a sublist"
        );

        let subsublist = sublist[0][1].as_list().unwrap();
        assert_eq!(
            subsublist[0][0].as_paragraph().unwrap().to_string(),
            "With another sublist"
        );

        let subsubsublist = subsublist[0][1].as_list().unwrap();
        assert_eq!(
            subsubsublist[0][0].as_paragraph().unwrap().to_string(),
            "And an additional sublist"
        );

        assert_eq!(
            subsublist[0][2].as_paragraph().unwrap().to_string(),
            "with content from the a sublist"
        );
        assert_eq!(
            sublist[0][2].as_paragraph().unwrap().to_string(),
            "and content after that sublist as paragraph within list item"
        );
    }

    #[test]
    fn list_should_properly_read_through_different_types_that_are_nested() {
        let input = Span::from(indoc! {"
            - nested
             * list
              1. of
               a) content
                second line of a
               second line of 1
              second line of bullet
             second line of hyphen
            * different list item
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 2, "Unexpected number of list items");

        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "nested");

        let sublist = l[0][1].as_list().unwrap();
        assert_eq!(sublist[0][0].as_paragraph().unwrap().to_string(), "list");

        let subsublist = sublist[0][1].as_list().unwrap();
        assert_eq!(subsublist[0][0].as_paragraph().unwrap().to_string(), "of");

        let subsubsublist = subsublist[0][1].as_list().unwrap();
        assert_eq!(
            subsubsublist[0][0].as_paragraph().unwrap().to_string(),
            "content\nsecond line of a"
        );

        assert_eq!(
            subsublist[0][2].as_paragraph().unwrap().to_string(),
            "second line of 1"
        );

        assert_eq!(
            sublist[0][2].as_paragraph().unwrap().to_string(),
            "second line of bullet"
        );

        assert_eq!(
            l[0][2].as_paragraph().unwrap().to_string(),
            "second line of hyphen"
        );

        assert_eq!(
            l[1][0].as_paragraph().unwrap().to_string(),
            "different list item"
        );
    }

    #[test]
    fn list_should_support_todo_list_items() {
        let input = Span::from(indoc! {"
            - [ ] list item 1
            - [.] list item 2
            - [o] list item 3
            - [O] list item 4
            - [X] list item 5
            - [-] list item 6
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list");
        assert_eq!(l.len(), 6, "Unexpected number of list items");

        assert!(l[0].is_todo_incomplete());
        assert_eq!(l[0][0].as_paragraph().unwrap().to_string(), "list item 1");

        assert!(l[1].is_todo_partially_complete_1());
        assert_eq!(l[1][0].as_paragraph().unwrap().to_string(), "list item 2");

        assert!(l[2].is_todo_partially_complete_2());
        assert_eq!(l[2][0].as_paragraph().unwrap().to_string(), "list item 3");

        assert!(l[3].is_todo_partially_complete_3());
        assert_eq!(l[3][0].as_paragraph().unwrap().to_string(), "list item 4");

        assert!(l[4].is_todo_complete());
        assert_eq!(l[4][0].as_paragraph().unwrap().to_string(), "list item 5");

        assert!(l[5].is_todo_rejected());
        assert_eq!(l[5][0].as_paragraph().unwrap().to_string(), "list item 6");
    }
}
