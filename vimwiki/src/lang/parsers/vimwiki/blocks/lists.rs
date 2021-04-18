use crate::lang::{
    elements::{
        InlineElementContainer, List, ListItem, ListItemAttributes,
        ListItemContent, ListItemContents, ListItemSuffix, ListItemTodoStatus,
        ListItemType, Located, OrderedListItemType, UnorderedListItemType,
    },
    parsers::{
        utils::{
            beginning_of_line, capture, context, deeper, end_of_line_or_input,
            locate,
        },
        vimwiki::blocks::inline::inline_element_container,
        IResult, Span,
    },
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, one_of, space0},
    combinator::{map, opt, peek, value, verify},
    multi::{fold_many0, many0, many1},
    sequence::{pair, preceded, terminated},
};

#[inline]
pub fn list(input: Span) -> IResult<Located<List>> {
    fn inner(input: Span) -> IResult<List> {
        // A list must at least have one item, whose indentation level we will
        // use to determine how far to go
        let (input, (indentation, item)) = deeper(list_item)(input)?;

        // TODO: Keep track of indentation level for a list based on its first
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

        // 3. Ensure that the item starts with a valid prefix
        let (input, item) = locate(capture(map(
            pair(list_item_prefix, list_item_tail(indentation)),
            |((item_type, item_suffix), (attrs, contents))| {
                // NOTE: To make things easier, we aren't assigning the index
                //       within this parser; rather, we put a filler index and
                //       will assign the actual index in the parent parser
                ListItem::new(item_type, item_suffix, 0, contents, attrs)
            },
        )))(input)?;

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
        let (input, content) = map(deeper(list_item_line_content), |c| {
            c.map(ListItemContent::from)
        })(input)?;

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
            alt((
                map(deeper(list), |c| c.map(ListItemContent::from)),
                map(preceded(space0, deeper(list_item_line_content)), |c| {
                    c.map(ListItemContent::from)
                }),
            )),
        ))(input)?;

        contents.insert(0, content);

        Ok((
            input,
            (
                ListItemAttributes {
                    todo_status: maybe_todo_status,
                },
                contents.into(),
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

/// Parses a line AFTER indentation has been parsed, treating the line as
/// a series of content.
#[inline]
fn list_item_line_content(
    input: Span,
) -> IResult<Located<InlineElementContainer>> {
    terminated(inline_element_container, end_of_line_or_input)(input)
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
/// aaa) Some other list item
/// AAA) Some other list item
/// iii) Some other list item
/// III) Some other list item
/// # Some other list item
///
#[inline]
fn ordered_list_item_prefix(
    input: Span,
) -> IResult<(OrderedListItemType, ListItemSuffix)> {
    // NOTE: Roman numeral check comes before alphabetic as alphabetic would
    //       also match roman numerals
    let (input, (item_type, item_suffix)) = alt((
        pair(ordered_list_item_type_number, list_item_suffix_period),
        pair(ordered_list_item_type_number, list_item_suffix_paren),
        pair(ordered_list_item_type_lower_roman, list_item_suffix_paren),
        pair(ordered_list_item_type_upper_roman, list_item_suffix_paren),
        pair(
            ordered_list_item_type_lower_alphabet,
            list_item_suffix_paren,
        ),
        pair(
            ordered_list_item_type_upper_alphabet,
            list_item_suffix_paren,
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
    use crate::lang::elements::{
        DecoratedText, DecoratedTextContent, InlineElement, Keyword, Link,
        MathInline, Tags, Text, WikiLink,
    };
    use indoc::indoc;
    use std::path::PathBuf;

    fn check_single_line_list_item(
        l: &List,
        item_type: ListItemType,
        item_suffix: ListItemSuffix,
        text: &str,
    ) {
        let item = &l.items[0].as_inner();
        assert_eq!(item.item_type, item_type);
        assert_eq!(item.suffix, item_suffix);
        assert_eq!(item.pos, 0);

        let element = match &item.contents[0].as_inner() {
            ListItemContent::InlineContent(c) => c.elements[0].as_inner(),
            x => panic!("Unexpected list item content: {:?}", x),
        };
        assert_eq!(element, &InlineElement::Text(Text::from(text)));
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
        for item in lst.items.iter() {
            assert_eq!(item.depth(), 1, "List item depth was at wrong level");
            for content in item.contents.contents.iter() {
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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

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
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            l.as_inner(),
            ListItemType::from(OrderedListItemType::UppercaseRoman),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_support_list_item_with_inline_content() {
        let input = Span::from(indoc! {r#"
            - list *item 1* has a [[link]] with :tag: and $formula$ is DONE
        "#});
        let (input, l) = list(input).unwrap();
        assert!(input.is_empty(), "Did not consume list item");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineElement>>(),
            vec![
                &InlineElement::Text(Text::from("list ")),
                &InlineElement::DecoratedText(DecoratedText::Bold(vec![
                    Located::from(DecoratedTextContent::from(Text::from(
                        "item 1"
                    )))
                ])),
                &InlineElement::Text(Text::from(" has a ")),
                &InlineElement::Link(Link::from(WikiLink::from(
                    PathBuf::from("link")
                ))),
                &InlineElement::Text(Text::from(" with ")),
                &InlineElement::Tags(Tags::from("tag")),
                &InlineElement::Text(Text::from(" and ")),
                &InlineElement::Math(MathInline::from("formula")),
                &InlineElement::Text(Text::from(" is ")),
                &InlineElement::Keyword(Keyword::Done),
            ]
        );
    }

    #[test]
    fn list_should_support_list_item_with_multiple_lines_of_content() {
        let input = Span::from(indoc! {"
            - list item 1
              has extra content
              on multiple lines
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(input.as_unsafe_remaining_str(), "not a list item\n");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineElement>>(),
            vec![
                &InlineElement::Text(Text::from("list item 1")),
                &InlineElement::Text(Text::from("has extra content")),
                &InlineElement::Text(Text::from("on multiple lines")),
            ]
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
        assert_eq!(input.as_unsafe_remaining_str(), "not a list item\n");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        // Should only have three lines of inline content
        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineElement>>(),
            vec![
                &InlineElement::Text(Text::from("list item 1")),
                &InlineElement::Text(Text::from("has extra content")),
                &InlineElement::Text(Text::from("on multiple lines")),
            ]
        );

        // Should have a single sublist with two items and content
        let sublist = l.items[0].contents.sublist_iter().next().unwrap();
        assert_eq!(sublist.items.len(), 2, "Unexpected number of list items");

        assert_eq!(
            sublist.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineElement>>(),
            vec![
                &InlineElement::Text(Text::from("sublist item 1")),
                &InlineElement::Text(Text::from("has content")),
            ]
        );

        assert_eq!(
            sublist.items[1]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineElement>>(),
            vec![&InlineElement::Text(Text::from("sublist item 2")),]
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
        assert_eq!(l.items.len(), 6, "Unexpected number of list items");

        assert!(l.items[0].is_todo_incomplete());
        assert_eq!(
            l.items[0].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 1"))),
        );

        assert!(l.items[1].is_todo_partially_complete_1());
        assert_eq!(
            l.items[1].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 2"))),
        );

        assert!(l.items[2].is_todo_partially_complete_2());
        assert_eq!(
            l.items[2].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 3"))),
        );

        assert!(l.items[3].is_todo_partially_complete_3());
        assert_eq!(
            l.items[3].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 4"))),
        );

        assert!(l.items[4].is_todo_complete());
        assert_eq!(
            l.items[4].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 5"))),
        );

        assert!(l.items[5].is_todo_rejected());
        assert_eq!(
            l.items[5].contents.inline_content_iter().next(),
            Some(&InlineElement::Text(Text::from("list item 6"))),
        );
    }
}
