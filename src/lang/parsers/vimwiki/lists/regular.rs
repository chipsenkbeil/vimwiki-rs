use super::{
    components::{
        EnhancedListItem, EnhancedListItemAttribute, InlineComponentContainer,
        ListItem, ListItemContent, ListItemContents, OrderedListItem,
        OrderedListItemSuffix, OrderedListItemType, RegularList,
        UnorderedListItem, UnorderedListItemType,
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
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
};

#[inline]
pub fn regular_list(input: Span) -> VimwikiIResult<LC<RegularList>> {
    let (input, pos) = position(input)?;

    // A list must at least have one item, whose indentation level we will
    // use to determine how far to go
    let (input, (indentation, item)) = list_item(0)(input)?;

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
    let mut items = vec![item];
    let mut index: usize = 1;
    while let Ok((input, item)) = preceded(
        verify(indentation_level(false), |level| *level == indentation),
        map(list_item(index), |x| x.1),
    )(input)
    {
        items.push(item);
        index += 1;
    }

    Ok((input, LC::from((RegularList::new(items), pos, input))))
}

/// Parse space/tabs before a list item, followed by the list item
#[inline]
fn list_item(
    index: usize,
) -> impl Fn(Span) -> VimwikiIResult<(usize, LC<EnhancedListItem>)> {
    move |input: Span| {
        // 1. Start at the beginning of the line
        let (input, _) = beginning_of_line(input)?;

        // 2. Determine the indentation level of this list item
        let (input, indentation) = indentation_level(true)(input)?;

        // 3. Ensure that the item starts with a valid prefix
        let (input, pos) = position(input)?;
        let (input, item) = alt((
            map(
                pair(ordered_list_item_prefix, list_item_tail(indentation)),
                |((item_type, item_suffix), (maybe_attr, contents))| {
                    // Construct list item based on results
                    let item = ListItem::from(OrderedListItem::new(
                        item_type,
                        item_suffix,
                        index,
                        contents,
                    ));
                    match maybe_attr {
                        Some(attr) => {
                            EnhancedListItem::new_with_attr(item, attr)
                        }
                        None => EnhancedListItem::from(item),
                    }
                },
            ),
            map(
                pair(unordered_list_item_prefix, list_item_tail(indentation)),
                |(item_type, (maybe_attr, contents))| {
                    // Construct list item based on results
                    let item = ListItem::from(UnorderedListItem::new(
                        item_type, index, contents,
                    ));
                    match maybe_attr {
                        Some(attr) => {
                            EnhancedListItem::new_with_attr(item, attr)
                        }
                        None => EnhancedListItem::from(item),
                    }
                },
            ),
        ))(input)?;

        Ok((input, (indentation, LC::from((item, pos, input)))))
    }
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
                map(preceded(space0, list_item_line_content), |c| {
                    c.map(ListItemContent::from)
                }),
                map(regular_list, |c| c.map(ListItemContent::from)),
            )),
        ))(input)?;

        contents.insert(0, content);

        Ok((input, (maybe_attr, contents)))
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
) -> VimwikiIResult<UnorderedListItemType> {
    alt((unordered_item_type_hyphen, unordered_item_type_asterisk))(input)
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
) -> VimwikiIResult<(OrderedListItemType, OrderedListItemSuffix)> {
    let (input, (item_type, item_suffix)) = alt((
        pair(ordered_item_type_number, ordered_item_suffix_period),
        pair(ordered_item_type_number, ordered_item_suffix_paren),
        pair(ordered_item_type_lower_alphabet, ordered_item_suffix_paren),
        pair(ordered_item_type_upper_alphabet, ordered_item_suffix_paren),
        pair(ordered_item_type_lower_roman, ordered_item_suffix_paren),
        pair(ordered_item_type_upper_roman, ordered_item_suffix_paren),
        pair(ordered_item_type_pound, ordered_item_suffix_none),
    ))(input)?;

    Ok((input, (item_type, item_suffix)))
}

#[inline]
fn unordered_item_type_hyphen(
    input: Span,
) -> VimwikiIResult<UnorderedListItemType> {
    value(UnorderedListItemType::Hyphen, tag("- "))(input)
}

#[inline]
fn unordered_item_type_asterisk(
    input: Span,
) -> VimwikiIResult<UnorderedListItemType> {
    value(UnorderedListItemType::Hyphen, tag("* "))(input)
}

#[inline]
fn ordered_item_type_number(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(OrderedListItemType::Number, digit1)(input)
}

#[inline]
fn ordered_item_type_pound(input: Span) -> VimwikiIResult<OrderedListItemType> {
    value(OrderedListItemType::Pound, tag("#"))(input)
}

#[inline]
fn ordered_item_type_lower_alphabet(
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
fn ordered_item_type_upper_alphabet(
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
fn ordered_item_type_lower_roman(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::LowercaseRoman,
        many1(one_of("ivxlcdm")),
    )(input)
}

#[inline]
fn ordered_item_type_upper_roman(
    input: Span,
) -> VimwikiIResult<OrderedListItemType> {
    value(
        OrderedListItemType::UppercaseRoman,
        many1(one_of("IVXLCDM")),
    )(input)
}

#[inline]
fn ordered_item_suffix_paren(
    input: Span,
) -> VimwikiIResult<OrderedListItemSuffix> {
    value(OrderedListItemSuffix::Paren, tag(") "))(input)
}

#[inline]
fn ordered_item_suffix_period(
    input: Span,
) -> VimwikiIResult<OrderedListItemSuffix> {
    value(OrderedListItemSuffix::Period, tag(". "))(input)
}

#[inline]
fn ordered_item_suffix_none(
    input: Span,
) -> VimwikiIResult<OrderedListItemSuffix> {
    value(OrderedListItemSuffix::None, tag(" "))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_should_fail_if_not_at_beginning_of_line() {
        todo!();
    }

    #[test]
    fn divider_should_fail_if_not_at_least_four_hyphens() {
        todo!();
    }

    #[test]
    fn divider_should_fail_if_not_only_hyphens_within_line() {
        todo!();
    }

    #[test]
    fn divider_should_succeed_if_four_or_more_hyphens_at_start_of_line() {
        todo!();
    }
}
