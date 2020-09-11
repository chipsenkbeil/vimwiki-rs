use super::{
    components::{
        EnhancedListItem, EnhancedListItemAttribute, InlineComponentContainer,
        List, ListItem, ListItemContent, ListItemContents, ListItemSuffix,
        ListItemType, OrderedListItemType, UnorderedListItemType,
    },
    inline_component_container,
    utils::{beginning_of_line, end_of_line_or_input, position},
    Span, VimwikiIResult, LC,
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
pub fn list(input: Span) -> VimwikiIResult<LC<List>> {
    let (input, pos) = position(input)?;

    // A list must at least have one item, whose indentation level we will
    // use to determine how far to go
    let (input, (indentation, item)) = list_item(input)?;

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
            map(list_item, |x| x.1),
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
    let l = List::new(items).normalize().to_owned();

    Ok((input, LC::from((l, pos, input))))
}

/// Parse space/tabs before a list item, followed by the list item
#[inline]
fn list_item(input: Span) -> VimwikiIResult<(usize, LC<EnhancedListItem>)> {
    // 1. Start at the beginning of the line
    let (input, _) = beginning_of_line(input)?;

    // 2. Determine the indentation level of this list item
    let (input, indentation) = indentation_level(true)(input)?;

    // 3. Ensure that the item starts with a valid prefix
    let (input, pos) = position(input)?;
    let (input, item) = map(
        pair(list_item_prefix, list_item_tail(indentation)),
        |((item_type, item_suffix), (maybe_attr, contents))| {
            // NOTE: To make things easier, we aren't assigning the index
            //       within this parser; rather, we put a filler index and
            //       will assign the actual index in the parent parser
            let item = ListItem::new(item_type, item_suffix, 0, contents);
            match maybe_attr {
                Some(attr) => EnhancedListItem::new_with_attr(item, attr),
                None => EnhancedListItem::from(item),
            }
        },
    )(input)?;

    Ok((input, (indentation, LC::from((item, pos, input)))))
}

#[inline]
fn list_item_tail(
    indentation: usize,
) -> impl Fn(
    Span,
)
    -> VimwikiIResult<(Option<EnhancedListItemAttribute>, ListItemContents)> {
    move |input: Span| {
        // 4. Check if we have a custom todo attribute
        let (input, maybe_attr) = opt(todo_attr)(input)?;

        // 5. Parse the rest of the current line
        let (input, content) =
            map(list_item_line_content, |c| c.map(ListItemContent::from))(
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
            alt((
                map(list, |c| c.map(ListItemContent::from)),
                map(preceded(space0, list_item_line_content), |c| {
                    c.map(ListItemContent::from)
                }),
            )),
        ))(input)?;

        contents.insert(0, content);

        Ok((input, (maybe_attr, contents.into())))
    }
}

/// Parser that determines the indentation level of the current line based
/// on its current position
#[inline]
fn indentation_level(consume: bool) -> impl Fn(Span) -> VimwikiIResult<usize> {
    move |input: Span| {
        if consume {
            map(space0, |s: Span| s.fragment().len())(input)
        } else {
            map(peek(space0), |s: Span| s.fragment().len())(input)
        }
    }
}

/// Parses a line AFTER indentation has been parsed, treating the line as
/// a series of content.
#[inline]
fn list_item_line_content(
    input: Span,
) -> VimwikiIResult<LC<InlineComponentContainer>> {
    terminated(inline_component_container, end_of_line_or_input)(input)
}

#[inline]
fn todo_attr(input: Span) -> VimwikiIResult<EnhancedListItemAttribute> {
    use EnhancedListItemAttribute as Attr;
    alt((
        value(Attr::TodoIncomplete, tag("[ ] ")),
        value(Attr::TodoPartiallyComplete1, tag("[.] ")),
        value(Attr::TodoPartiallyComplete2, tag("[o] ")),
        value(Attr::TodoPartiallyComplete3, tag("[O] ")),
        value(Attr::TodoComplete, tag("[X] ")),
        value(Attr::TodoRejected, tag("[-] ")),
    ))(input)
}

#[inline]
fn list_item_prefix(
    input: Span,
) -> VimwikiIResult<(ListItemType, ListItemSuffix)> {
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
///     - Some list item
///     * Some other list item
///
#[inline]
fn unordered_list_item_prefix(
    input: Span,
) -> VimwikiIResult<(UnorderedListItemType, ListItemSuffix)> {
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
///     1. Some list item
///     1) Some other list item
///     aaa) Some other list item
///     AAA) Some other list item
///     iii) Some other list item
///     III) Some other list item
///     # Some other list item
///
#[inline]
fn ordered_list_item_prefix(
    input: Span,
) -> VimwikiIResult<(OrderedListItemType, ListItemSuffix)> {
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
) -> VimwikiIResult<UnorderedListItemType> {
    value(UnorderedListItemType::Hyphen, tag("- "))(input)
}

#[inline]
fn unordered_list_item_type_asterisk(
    input: Span,
) -> VimwikiIResult<UnorderedListItemType> {
    value(UnorderedListItemType::Asterisk, tag("* "))(input)
}

#[inline]
fn ordered_list_item_type_number(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(OrderedListItemType::Number, digit1)(input)
}

#[inline]
fn ordered_list_item_type_pound(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(OrderedListItemType::Pound, tag("#"))(input)
}

#[inline]
fn ordered_list_item_type_lower_alphabet(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::LowercaseAlphabet,
        take_while1(|c: char| {
            c.is_ascii_alphabetic() && c.is_ascii_lowercase()
        }),
    )(input)
}

#[inline]
fn ordered_list_item_type_upper_alphabet(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::UppercaseAlphabet,
        take_while1(|c: char| {
            c.is_ascii_alphabetic() && c.is_ascii_uppercase()
        }),
    )(input)
}

#[inline]
fn ordered_list_item_type_lower_roman(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::LowercaseRoman,
        many1(one_of("ivxlcdm")),
    )(input)
}

#[inline]
fn ordered_list_item_type_upper_roman(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::UppercaseRoman,
        many1(one_of("IVXLCDM")),
    )(input)
}

#[inline]
fn list_item_suffix_paren(input: Span) -> VimwikiIResult<ListItemSuffix> {
    value(ListItemSuffix::Paren, tag(") "))(input)
}

#[inline]
fn list_item_suffix_period(input: Span) -> VimwikiIResult<ListItemSuffix> {
    value(ListItemSuffix::Period, tag(". "))(input)
}

#[inline]
fn list_item_suffix_none(input: Span) -> VimwikiIResult<ListItemSuffix> {
    value(ListItemSuffix::None, tag(" "))(input)
}

#[cfg(test)]
mod tests {
    use super::super::components::{
        DecoratedText, DecoratedTextContent, Decoration, InlineComponent,
        Keyword, Link, MathInline, Tags, WikiLink,
    };
    use super::*;
    use indoc::indoc;
    use std::path::PathBuf;

    fn check_single_line_list_item(
        l: &List,
        item_type: ListItemType,
        item_suffix: ListItemSuffix,
        text: &str,
    ) {
        let item = &l.items[0].component;
        assert_eq!(item.item_type, item_type);
        assert_eq!(item.suffix, item_suffix);
        assert_eq!(item.pos, 0);

        let component = match &item.contents[0].component {
            ListItemContent::InlineContent(c) => &c.components[0].component,
            x => panic!("Unexpected list item content: {:?}", x),
        };
        assert_eq!(component, &InlineComponent::Text(text.to_string()));
    }

    #[test]
    fn list_should_fail_if_no_proper_start_to_single_list_item() {
        let input = Span::new("| some item with bad prefix");
        assert!(list(input).is_err());
    }

    #[test]
    fn list_should_fail_if_no_space_after_single_list_item_prefix() {
        assert!(list(Span::new("-some item with no space")).is_err());
        assert!(list(Span::new("*some item with no space")).is_err());
        assert!(list(Span::new("1.some item with no space")).is_err());
        assert!(list(Span::new("1)some item with no space")).is_err());
        assert!(list(Span::new("a)some item with no space")).is_err());
        assert!(list(Span::new("A)some item with no space")).is_err());
        assert!(list(Span::new("i)some item with no space")).is_err());
        assert!(list(Span::new("I)some item with no space")).is_err());
        assert!(list(Span::new("#some item with no space")).is_err());
    }

    #[test]
    fn list_should_succeed_for_single_unordered_hyphen_item() {
        let input = Span::new("- list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_unordered_asterisk_item() {
        let input = Span::new("* list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(UnorderedListItemType::Asterisk),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_pound_item() {
        let input = Span::new("# list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::Pound),
            ListItemSuffix::None,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_number_period_item() {
        let input = Span::new("1. list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::Number),
            ListItemSuffix::Period,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_number_paren_item() {
        let input = Span::new("1) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::Number),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_lowercase_alphabet_paren_item() {
        let input = Span::new("a) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::LowercaseAlphabet),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_uppercase_alphabet_paren_item() {
        let input = Span::new("A) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::UppercaseAlphabet),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_lowercase_roman_paren_item() {
        let input = Span::new("i) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::LowercaseRoman),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_succeed_for_single_ordered_uppercase_roman_paren_item() {
        let input = Span::new("I) list item 1");
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        check_single_line_list_item(
            &l.component,
            ListItemType::from(OrderedListItemType::UppercaseRoman),
            ListItemSuffix::Paren,
            "list item 1",
        );
    }

    #[test]
    fn list_should_support_list_item_with_inline_content() {
        let input = Span::new(indoc! {r#"
            - list *item 1* has a [[link]] with :tag: and $formula$ is DONE
        "#});
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list item");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineComponent>>(),
            vec![
                &InlineComponent::Text("list ".to_string()),
                &InlineComponent::DecoratedText(DecoratedText::new(
                    vec![LC::from(DecoratedTextContent::Text(
                        "item 1".to_string()
                    ))],
                    Decoration::Bold
                )),
                &InlineComponent::Text(" has a ".to_string()),
                &InlineComponent::Link(Link::from(WikiLink::from(
                    PathBuf::from("link")
                ))),
                &InlineComponent::Text(" with ".to_string()),
                &InlineComponent::Tags(Tags::from("tag")),
                &InlineComponent::Text(" and ".to_string()),
                &InlineComponent::Math(MathInline::new("formula".to_string())),
                &InlineComponent::Text(" is ".to_string()),
                &InlineComponent::Keyword(Keyword::DONE),
            ]
        );
    }

    #[test]
    fn list_should_support_list_item_with_multiple_lines_of_content() {
        let input = Span::new(indoc! {"
            - list item 1
              has extra content
              on multiple lines
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(*input.fragment(), "not a list item\n");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineComponent>>(),
            vec![
                &InlineComponent::Text("list item 1".to_string()),
                &InlineComponent::Text("has extra content".to_string()),
                &InlineComponent::Text("on multiple lines".to_string()),
            ]
        );
    }

    #[test]
    fn list_should_support_list_item_with_content_separated_by_sublist() {
        let input = Span::new(indoc! {"
            - list item 1
              has extra content
              - sublist item 1
                has content
              - sublist item 2
              on multiple lines
            not a list item
        "});
        let (input, l) = list(input).unwrap();
        assert_eq!(*input.fragment(), "not a list item\n");
        assert_eq!(l.items.len(), 1, "Unexpected number of list items");

        // Should only have three lines of inline content
        assert_eq!(
            l.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineComponent>>(),
            vec![
                &InlineComponent::Text("list item 1".to_string()),
                &InlineComponent::Text("has extra content".to_string()),
                &InlineComponent::Text("on multiple lines".to_string()),
            ]
        );

        // Should have a single sublist with two items and content
        let sublist = l.items[0].contents.sublist_iter().next().unwrap();
        assert_eq!(sublist.items.len(), 2, "Unexpected number of list items");

        assert_eq!(
            sublist.items[0]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineComponent>>(),
            vec![
                &InlineComponent::Text("sublist item 1".to_string()),
                &InlineComponent::Text("has content".to_string()),
            ]
        );

        assert_eq!(
            sublist.items[1]
                .contents
                .inline_content_iter()
                .collect::<Vec<&InlineComponent>>(),
            vec![&InlineComponent::Text("sublist item 2".to_string()),]
        );
    }

    #[test]
    fn list_should_support_todo_list_items() {
        let input = Span::new(indoc! {"
            - [ ] list item 1
            - [.] list item 2
            - [o] list item 3
            - [O] list item 4
            - [X] list item 5
            - [-] list item 6
        "});
        let (input, l) = list(input).unwrap();
        assert!(input.fragment().is_empty(), "Did not consume list");
        assert_eq!(l.items.len(), 6, "Unexpected number of list items");

        assert!(l.items[0].is_todo_incomplete());
        assert_eq!(
            l.items[0].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 1".to_string())),
        );

        assert!(l.items[1].is_todo_partially_complete_1());
        assert_eq!(
            l.items[1].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 2".to_string())),
        );

        assert!(l.items[2].is_todo_partially_complete_2());
        assert_eq!(
            l.items[2].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 3".to_string())),
        );

        assert!(l.items[3].is_todo_partially_complete_3());
        assert_eq!(
            l.items[3].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 4".to_string())),
        );

        assert!(l.items[4].is_todo_complete());
        assert_eq!(
            l.items[4].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 5".to_string())),
        );

        assert!(l.items[5].is_todo_rejected());
        assert_eq!(
            l.items[5].contents.inline_content_iter().next(),
            Some(&InlineComponent::Text("list item 6".to_string())),
        );
    }
}
