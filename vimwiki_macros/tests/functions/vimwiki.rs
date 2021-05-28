use std::{borrow::Cow, convert::TryFrom};
use vimwiki::{
    vendor::{chrono::NaiveDate, uriparse::URIReference},
    *,
};
use vimwiki_macros::*;

#[test]
fn vimwiki_page() {
    let x = vimwiki_page!("some text");
    assert_eq!(
        x,
        Page::new(vec![Located::from(BlockElement::Paragraph(
            Paragraph::new(vec![InlineElementContainer::new(vec![
                Located::from(InlineElement::Text(Text::from("some text")))
            ])])
        ))],)
    );
}

#[test]
fn vimwiki_block_element() {
    let x = vimwiki_block_element!("some text");
    assert_eq!(
        x.into_inner(),
        BlockElement::Paragraph(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::Text(Text::from("some text"))
            )])
        ]))
    );
}

#[test]
fn vimwiki_inline_element_container() {
    let x = vimwiki_inline_element_container!("some text");
    assert_eq!(
        x.into_inner(),
        InlineElementContainer::new(vec![Located::from(InlineElement::Text(
            Text::from("some text")
        ))])
    );
}

#[test]
fn vimwiki_inline_element() {
    let x = vimwiki_inline_element!("some text");
    assert_eq!(x.into_inner(), InlineElement::Text(Text::from("some text")));
}

#[test]
fn vimwiki_blockquote() {
    let x = vimwiki_blockquote!("> some text");
    assert_eq!(
        x.into_inner(),
        Blockquote::new(vec![Cow::from("some text")])
    );
}

#[test]
fn vimwiki_comment() {
    let x = vimwiki_comment!("%% some comment");
    assert_eq!(
        x.into_inner(),
        Comment::from(LineComment::new(Cow::from(" some comment")))
    );
}

#[test]
fn vimwiki_line_comment() {
    let x = vimwiki_line_comment!("%% some comment");
    assert_eq!(x.into_inner(), LineComment::new(Cow::from(" some comment")));
}

#[test]
fn vimwiki_multi_line_comment() {
    let x = vimwiki_multi_line_comment!("%%+ some comment +%%");
    assert_eq!(
        x.into_inner(),
        MultiLineComment::new(vec![Cow::from(" some comment ")])
    );
}

#[test]
fn vimwiki_definition_list() {
    let x = vimwiki_definition_list! {r#"
        term:: definition
        term2::
        :: def 2
        :: def 3
    "#};
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
                        InlineElement::from(Text::from("definition"))
                    )])
                ))],
            ),
            (
                Located::from(DefinitionListValue::new(
                    InlineElementContainer::new(vec![Located::from(
                        InlineElement::from(Text::from("term2"))
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
                            InlineElement::from(Text::from("def 3"))
                        )])
                    ))
                ],
            )
        ])
    );
}

#[test]
fn vimwiki_divider() {
    let x = vimwiki_divider!("----");
    assert_eq!(x.into_inner(), Divider);
}

#[test]
fn vimwiki_header() {
    let x = vimwiki_header!("= header =");
    assert_eq!(
        x.into_inner(),
        Header::new(
            1,
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("header"))
            )]),
            false
        )
    );
}

#[test]
fn vimwiki_link() {
    // Wiki Link
    let x = vimwiki_link!("[[link]]");
    assert_eq!(
        x.into_inner(),
        Link::new_wiki_link(
            URIReference::try_from("link").unwrap().into_owned(),
            None
        ),
    );

    // Indexed Interwiki Link
    let x = vimwiki_link!("[[wiki1:link]]");
    assert_eq!(
        x.into_inner(),
        Link::new_indexed_interwiki_link(
            1,
            URIReference::try_from("link").unwrap().into_owned(),
            None
        ),
    );

    // Named Interwiki Link
    let x = vimwiki_link!("[[wn.MyWiki:link]]");
    assert_eq!(
        x.into_inner(),
        Link::new_named_interwiki_link(
            "MyWiki",
            URIReference::try_from("link").unwrap().into_owned(),
            None
        ),
    );

    // Diary Link
    let x = vimwiki_link!("[[diary:2012-03-05]]");
    assert_eq!(
        x.into_inner(),
        Link::new_diary_link(NaiveDate::from_ymd(2012, 3, 5), None, None)
    );

    // File Link
    let x = vimwiki_link!("[[file:path/to/file]]");
    assert_eq!(
        x.into_inner(),
        Link::new_wiki_link(
            URIReference::try_from("file:path/to/file")
                .unwrap()
                .into_owned(),
            None,
        )
    );

    // Raw Link
    let x = vimwiki_link!("https://example.com");
    assert_eq!(
        x.into_inner(),
        Link::new_raw_link(
            URIReference::try_from("https://example.com")
                .unwrap()
                .into_owned()
        )
    );

    // Transclusion Link
    let x = vimwiki_link!("{{https://example.com/img.jpg}}");
    assert_eq!(
        x.into_inner(),
        Link::new_transclusion_link(
            URIReference::try_from("https://example.com/img.jpg")
                .unwrap()
                .into_owned(),
            None,
            None,
        ),
    );
}

#[test]
fn vimwiki_list() {
    let x = vimwiki_list! {"
        - some list item
        - some other list item
            1. sub list item
    "};
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
                            InlineElement::Text(Text::from("some list item"))
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
    let x = vimwiki_list_item!("- some list item");
    assert_eq!(
        x.into_inner(),
        ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(InlineElementContainer::new(
                    vec![Located::from(InlineElement::Text(Text::from(
                        "some list item"
                    )))]
                ))
            )]),
            ListItemAttributes { todo_status: None }
        )
    );
}

#[test]
fn vimwiki_list_raw() {
    let x = vimwiki_list_raw!("- some list item");
    assert_eq!(
        x.into_inner(),
        List::new(vec![Located::from(ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![Located::from(
                ListItemContent::InlineContent(InlineElementContainer::new(
                    vec![Located::from(InlineElement::Text(Text::from(
                        "some list item"
                    )))]
                ))
            )]),
            ListItemAttributes::default(),
        ))])
    );
}

#[test]
fn vimwiki_code_inline() {
    let x = vimwiki_code_inline!("`code`");
    assert_eq!(x.into_inner(), CodeInline::new(Cow::from("code")));
}

#[test]
fn vimwiki_math_inline() {
    let x = vimwiki_math_inline!("$math$");
    assert_eq!(x.into_inner(), MathInline::new(Cow::from("math")));
}

#[test]
fn vimwiki_math_block() {
    let x = vimwiki_math_block! {r#"
    {{$
    math
    }}$
    "#};
    assert_eq!(
        x.into_inner(),
        MathBlock::new(vec![Cow::from("math")], None)
    );
}

#[test]
fn vimwiki_math_block_raw() {
    let x = vimwiki_math_block_raw! {r#"{{$
    math
    }}$"#};
    assert_eq!(
        x.into_inner(),
        MathBlock::new(vec![Cow::from("    math")], None)
    );
}

#[test]
fn vimwiki_paragraph() {
    let x = vimwiki_paragraph!("some text");
    assert_eq!(
        x.into_inner(),
        Paragraph::new(vec![InlineElementContainer::new(vec![Located::from(
            InlineElement::Text(Text::from("some text"))
        )])])
    );
}

#[test]
fn vimwiki_placeholder() {
    assert_eq!(
        vimwiki_placeholder!("%date 2012-03-05").into_inner(),
        Placeholder::Date(NaiveDate::from_ymd(2012, 3, 5)),
    );
    assert_eq!(
        vimwiki_placeholder!("%nohtml").into_inner(),
        Placeholder::NoHtml,
    );
    assert_eq!(
        vimwiki_placeholder!("%other some text").into_inner(),
        Placeholder::Other {
            name: Cow::from("other"),
            value: Cow::from("some text")
        },
    );
    assert_eq!(
        vimwiki_placeholder!("%template my_template").into_inner(),
        Placeholder::Template(Cow::from("my_template")),
    );
    assert_eq!(
        vimwiki_placeholder!("%title some text").into_inner(),
        Placeholder::Title(Cow::from("some text")),
    );
}

#[test]
fn vimwiki_code_block() {
    let x = vimwiki_code_block! {r#"
    {{{
    some code
    }}}
    "#};
    assert_eq!(
        x.into_inner(),
        CodeBlock::new(None, Default::default(), vec![Cow::from("some code")])
    );
}

#[test]
fn vimwiki_code_block_raw() {
    let x = vimwiki_code_block_raw! {r#"{{{
    some code
    }}}"#};
    assert_eq!(
        x.into_inner(),
        CodeBlock::new(
            None,
            Default::default(),
            vec![Cow::from("    some code")]
        )
    );
}

#[test]
fn vimwiki_table() {
    let x = vimwiki_table!("|cell|");
    assert_eq!(
        x.into_inner(),
        Table::new(
            vec![(
                CellPos::new(0, 0),
                Located::from(Cell::Content(InlineElementContainer::new(
                    vec![Located::from(InlineElement::Text(Text::from(
                        "cell"
                    )))]
                )))
            )],
            false
        )
    );
}

#[test]
fn vimwiki_tags() {
    let x = vimwiki_tags!(":tag:");
    assert_eq!(x.into_inner(), Tags::from("tag"));
}

#[test]
fn vimwiki_text() {
    let x = vimwiki_text!("some text");
    assert_eq!(x.into_inner(), Text::from("some text"));
}

#[test]
fn vimwiki_decorated_text_bold() {
    let x = vimwiki_decorated_text!("*some text*");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Bold(vec![Located::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_italic() {
    let x = vimwiki_decorated_text!("_some text_");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Italic(vec![Located::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_strikeout() {
    let x = vimwiki_decorated_text!("~~some text~~");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Strikeout(vec![Located::from(
            DecoratedTextContent::from(Text::from("some text"))
        )],)
    );
}

#[test]
fn vimwiki_decorated_text_superscript() {
    let x = vimwiki_decorated_text!("^some text^");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Superscript(vec![Located::from(
            DecoratedTextContent::from(Text::from("some text"))
        )],)
    );
}

#[test]
fn vimwiki_decorated_text_subscript() {
    let x = vimwiki_decorated_text!(",,some text,,");
    assert_eq!(
        x.into_inner(),
        DecoratedText::Subscript(vec![Located::from(
            DecoratedTextContent::from(Text::from("some text"))
        )],)
    );
}

#[test]
fn vimwiki_keyword() {
    let x = vimwiki_keyword!("TODO");
    assert_eq!(x.into_inner(), Keyword::Todo);
}
