use std::{borrow::Cow, convert::TryFrom};
use vimwiki::{
    vendor::{chrono::NaiveDate, uriparse::URIReference},
    *,
};
use vimwiki_macros::*;

#[test]
fn vimwiki_page() {
    let x = vimwiki_page_format!("some {} text", "cool");
    assert_eq!(
        x,
        Page::new(vec![Located::from(BlockElement::Paragraph(
            Paragraph::new(vec![InlineElementContainer::new(vec![
                Located::from(InlineElement::Text(Text::from(
                    "some cool text"
                )))
            ])])
        ))])
    );
}

#[test]
fn vimwiki_block_element() {
    let x = vimwiki_block_element_format!("some {} text", "cool");
    assert_eq!(
        x.into_inner(),
        BlockElement::Paragraph(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::Text(Text::from("some cool text"))
            )])
        ]))
    );
}

#[test]
fn vimwiki_inline_element_container() {
    let x = vimwiki_inline_element_container_format!("some {} text", "cool");
    assert_eq!(
        x.into_inner(),
        InlineElementContainer::new(vec![Located::from(InlineElement::Text(
            Text::from("some cool text")
        ))])
    );
}

#[test]
fn vimwiki_inline_element() {
    let x = vimwiki_inline_element_format!("some {} text", "cool");
    assert_eq!(
        x.into_inner(),
        InlineElement::Text(Text::from("some cool text"))
    );
}

#[test]
fn vimwiki_blockquote() {
    let x = vimwiki_blockquote_format!("> some {} text", "cool");
    assert_eq!(
        x.into_inner(),
        Blockquote::new(vec![Cow::from("some cool text")])
    );
}

#[test]
fn vimwiki_comment() {
    let x = vimwiki_comment_format!("%% some {} comment", "cool");
    assert_eq!(
        x.into_inner(),
        Comment::from(LineComment::new(Cow::from(" some cool comment")))
    );
}

#[test]
fn vimwiki_line_comment() {
    let x = vimwiki_line_comment_format!("%% some {} comment", "cool");
    assert_eq!(
        x.into_inner(),
        LineComment::new(Cow::from(" some cool comment"))
    );
}

#[test]
fn vimwiki_multi_line_comment() {
    let x =
        vimwiki_multi_line_comment_format!("%%+ some {} comment +%%", "cool");
    assert_eq!(
        x.into_inner(),
        MultiLineComment::new(vec![Cow::from(" some cool comment ")])
    );
}

#[test]
fn vimwiki_definition_list() {
    let x = vimwiki_definition_list_format!(
        r#"
            term:: {} definition
            term2 {}::
            :: def 2
            :: def {} 3
        "#,
        "first",
        "second",
        "third",
    );
    assert_eq!(
        x.into_inner(),
        DefinitionList::from(vec![
            (
                Located::from(DefinitionListValue::new(
                    InlineElementContainer::new(vec![Located::from(
                        InlineElement::from(Text::from("term"))
                    )])
                )),
                vec![Located::from(DefinitionListValue::new(
                    InlineElementContainer::new(vec![Located::from(
                        InlineElement::from(Text::from("first definition"))
                    )])
                ))],
            ),
            (
                Located::from(DefinitionListValue::new(
                    InlineElementContainer::new(vec![Located::from(
                        InlineElement::from(Text::from("term2 second"))
                    )])
                )),
                vec![
                    Located::from(DefinitionListValue::new(
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::from(Text::from("def 2"))
                        )])
                    )),
                    Located::from(DefinitionListValue::new(
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::from(Text::from("def third 3"))
                        )])
                    ))
                ],
            )
        ])
    );
}

#[test]
fn vimwiki_header() {
    let x = vimwiki_header_format!("= {} header =", "cool");
    assert_eq!(
        x.into_inner(),
        Header::new(
            1,
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("cool header"))
            )]),
            false
        )
    );
}

#[test]
fn vimwiki_link() {
    // Wiki link (NOTE: Cannot inject into uri, only description)
    let x = vimwiki_link_format!("[[link|{}]]", "cool");
    assert_eq!(
        x.into_inner(),
        Link::new_wiki_link(
            URIReference::try_from("link").unwrap().into_owned(),
            Description::from("cool"),
        ),
    );

    // Indexed Interwiki Link (NOTE: Cannot inject into uri, only description)
    let x = vimwiki_link_format!("[[wiki1:link|{}]]", "cool");
    assert_eq!(
        x.into_inner(),
        Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("cool%20link").unwrap().into_owned(),
            Description::from("cool"),
        ),
    );

    // Named Interwiki Link (NOTE: Cannot inject into uri, only description)
    let x = vimwiki_link_format!("[[wn.MyWiki:link|{}]]", "cool");
    assert_eq!(
        x.into_inner(),
        Link::new_named_interwiki_link(
            "MyWiki",
            URIReference::try_from("cool%20link").unwrap().into_owned(),
            Description::from("cool"),
        ),
    );

    // Diary link (NOTE: Cannot inject into date, only description)
    let x = vimwiki_link_format!("[[diary:2012-03-05|{}]]", "cool");
    assert_eq!(
        x.into_inner(),
        Link::new_diary_link(
            NaiveDate::from_ymd(2012, 3, 5),
            Description::from("cool"),
        )
    );

    // File link (NOTE: Cannot inject into uri, only description)
    let x = vimwiki_link_format!("[[file:path/to/file|{}]]", "cool");
    assert_eq!(
        x.into_inner(),
        Link::new_wiki_link(
            URIReference::try_from("file:path/to/file")
                .unwrap()
                .into_owned(),
            Description::from("cool"),
        )
    );
}

#[test]
fn vimwiki_list() {
    let x = vimwiki_list_format!(
        r"
            - some {} list item
            - some other list item
                1. sub list item
        ",
        "cool",
    );
    assert_eq!(
        x.into_inner(),
        List::new(vec![
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                0,
                ListItemContents::new(vec![Located::from(
                    ListItemContent::InlineContent(
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::Text(Text::from(
                                "some cool list item"
                            ))
                        )])
                    )
                )]),
                ListItemAttributes::default(),
            ),),
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                1,
                ListItemContents::new(vec![
                    Located::from(ListItemContent::InlineContent(
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::Text(Text::from(
                                "some other list item"
                            ))
                        )])
                    )),
                    Located::from(ListItemContent::List(List::new(vec![
                        Located::from(ListItem::new(
                            ListItemType::Ordered(OrderedListItemType::Number),
                            ListItemSuffix::Period,
                            0,
                            ListItemContents::new(vec![Located::from(
                                ListItemContent::InlineContent(
                                    InlineElementContainer::new(vec![
                                        Located::from(InlineElement::Text(
                                            Text::from("sub list item")
                                        ))
                                    ])
                                )
                            )]),
                            ListItemAttributes::default(),
                        ))
                    ])))
                ]),
                ListItemAttributes::default(),
            ))
        ])
    );
}

#[test]
fn vimwiki_list_item() {
    let x = vimwiki_list_item_format!("- some {} list item", "cool");
    assert_eq!(
        x.into_inner(),
        ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(InlineElementContainer::new(
                    vec![Located::from(InlineElement::Text(Text::from(
                        "some cool list item"
                    )))]
                ))
            )]),
            ListItemAttributes { todo_status: None }
        )
    );
}

#[test]
fn vimwiki_code_inline() {
    let x = vimwiki_code_inline_format!("`{} code`", "cool");
    assert_eq!(x.into_inner(), CodeInline::new(Cow::from("cool code")));
}

#[test]
fn vimwiki_math_inline() {
    let x = vimwiki_math_inline_format!("${} math$", "cool");
    assert_eq!(x.into_inner(), MathInline::new(Cow::from("cool math")));
}

#[test]
fn vimwiki_math_block() {
    let x = vimwiki_math_block_format!(
        r#"
        {{$
        {}
        math
        }}$
        "#,
        "cool",
    );
    assert_eq!(
        x.into_inner(),
        MathBlock::new(vec![Cow::from("cool"), Cow::from("math")], None)
    );
}

#[test]
fn vimwiki_paragraph() {
    let x = vimwiki_paragraph_format!("some {} text", "cool");
    assert_eq!(
        x.into_inner(),
        Paragraph::new(vec![InlineElementContainer::new(vec![Located::from(
            InlineElement::Text(Text::from("some cool text"))
        )])])
    );
}

#[test]
fn vimwiki_placeholder() {
    assert_eq!(
        vimwiki_placeholder_format!("%other some {} text", "cool").into_inner(),
        Placeholder::Other {
            name: Cow::from("other"),
            value: Cow::from("some cool text")
        },
    );
    assert_eq!(
        vimwiki_placeholder_format!("%template my_{}_template", "cool")
            .into_inner(),
        Placeholder::Template(Cow::from("my_cool_template")),
    );
    assert_eq!(
        vimwiki_placeholder_format!("%title some {} text", "cool").into_inner(),
        Placeholder::Title(Cow::from("some cool text")),
    );
}

#[test]
fn vimwiki_preformatted_text() {
    let x = vimwiki_preformatted_text_format!(
        r#"
        {{{
        some {} code
        }}}
        "#,
        "cool"
    );
    assert_eq!(
        x.into_inner(),
        PreformattedText::new(
            None,
            Default::default(),
            vec![Cow::from("some cool code")]
        )
    );
}

#[test]
fn vimwiki_table() {
    let x = vimwiki_table_format!("|{} cell|", "cool");
    assert_eq!(
        x.into_inner(),
        Table::new(
            vec![(
                CellPos::new(0, 0),
                Located::from(Cell::Content(InlineElementContainer::new(
                    vec![Located::from(InlineElement::Text(Text::from(
                        "cool cell"
                    )))]
                )))
            )],
            false
        )
    );
}

#[test]
fn vimwiki_tags() {
    let x = vimwiki_tags_format!(":{}tag:", "cool");
    assert_eq!(x.into_inner(), Tags::from("cooltag"));
}

#[test]
fn vimwiki_text() {
    let x = vimwiki_text_format!("some {} text", "cool");
    assert_eq!(x.into_inner(), Text::from("some cool text"));
}

#[test]
fn vimwiki_decorated_text_bold() {
    let x = vimwiki_decorated_text_format!("*some {} text*", "cool");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Bold(vec![Located::from(DecoratedTextContent::from(
            Text::from("some cool text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_italic() {
    let x = vimwiki_decorated_text_format!("_some {} text_", "cool");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Italic(vec![Located::from(DecoratedTextContent::from(
            Text::from("some cool text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_strikeout() {
    let x = vimwiki_decorated_text_format!("~~some {} text~~", "cool");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Strikeout(vec![Located::from(
            DecoratedTextContent::from(Text::from("some cool text"))
        )],)
    );
}

#[test]
fn vimwiki_decorated_text_superscript() {
    let x = vimwiki_decorated_text_format!("^some {} text^", "cool");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Superscript(vec![Located::from(
            DecoratedTextContent::from(Text::from("some cool text"))
        )],)
    );
}

#[test]
fn vimwiki_decorated_text_subscript() {
    let x = vimwiki_decorated_text_format!(",,some {} text,,", "cool");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Subscript(vec![Located::from(
            DecoratedTextContent::from(Text::from("some cool text"))
        )],)
    );
}
