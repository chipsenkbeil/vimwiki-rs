use std::{convert::TryFrom, path::PathBuf};
use vimwiki::{
    elements::*,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LE,
};
use vimwiki_macros::*;

#[test]
fn vimwiki_page() {
    let x = vimwiki_page!("some text");
    assert_eq!(
        x.element,
        Page::new(
            vec![LE::from(BlockElement::Paragraph(Paragraph::new(
                InlineElementContainer::new(vec![LE::from(
                    InlineElement::Text(Text::from("some text"))
                )])
            )))],
            vec![]
        )
    );
}

#[test]
fn vimwiki_block_element() {
    let x = vimwiki_block_element!("some text");
    assert_eq!(
        x.element,
        BlockElement::Paragraph(Paragraph::new(InlineElementContainer::new(
            vec![LE::from(InlineElement::Text(Text::from("some text")))]
        )))
    );
}

#[test]
fn vimwiki_inline_element_container() {
    let x = vimwiki_inline_element_container!("some text");
    assert_eq!(
        x.element,
        InlineElementContainer::new(vec![LE::from(InlineElement::Text(
            Text::from("some text")
        ))])
    );
}

#[test]
fn vimwiki_inline_element() {
    let x = vimwiki_inline_element!("some text");
    assert_eq!(x.element, InlineElement::Text(Text::from("some text")));
}

#[test]
fn vimwiki_blockquote() {
    let x = vimwiki_blockquote!("> some text");
    assert_eq!(x.element, Blockquote::new(vec!["some text".to_string()]));
}

#[test]
fn vimwiki_comment() {
    let x = vimwiki_comment!("%% some comment");
    assert_eq!(
        x.element,
        Comment::from(LineComment(" some comment".to_string()))
    );
}

#[test]
fn vimwiki_line_comment() {
    let x = vimwiki_line_comment!("%% some comment");
    assert_eq!(x.element, LineComment(" some comment".to_string()));
}

#[test]
fn vimwiki_multi_line_comment() {
    let x = vimwiki_multi_line_comment!("%%+ some comment +%%");
    assert_eq!(
        x.element,
        MultiLineComment(vec![" some comment ".to_string()])
    );
}

#[test]
fn vimwiki_definition_list() {
    let x = vimwiki_definition_list!("term:: definition");
    assert_eq!(
        x.element,
        DefinitionList::from(vec![TermAndDefinitions::new(
            InlineElementContainer::new(vec![LE::from(InlineElement::from(
                Text::from("term")
            ))]),
            vec![InlineElementContainer::new(vec![LE::from(
                InlineElement::from(Text::from("definition"))
            )])],
        )])
    );
}

#[test]
fn vimwiki_divider() {
    let x = vimwiki_divider!("----");
    assert_eq!(x.element, Divider);
}

#[test]
fn vimwiki_header() {
    let x = vimwiki_header!("= header =");
    assert_eq!(
        x.element,
        Header::new(
            1,
            InlineElementContainer::new(vec![LE::from(InlineElement::from(
                Text::from("header")
            ))]),
            false
        )
    );
}

#[test]
fn vimwiki_link() {
    let x = vimwiki_link!("[[link]]");
    assert_eq!(x.element, Link::Wiki(WikiLink::from(PathBuf::from("link"))));
}

#[test]
fn vimwiki_diary_link() {
    let x = vimwiki_diary_link!("[[diary:2012-03-05]]");
    assert_eq!(x.element, DiaryLink::from(NaiveDate::from_ymd(2012, 3, 5)));
}

#[test]
fn vimwiki_external_file_link() {
    let x = vimwiki_external_file_link!("[[file:path/to/file]]");
    assert_eq!(
        x.element,
        ExternalFileLink::new(
            ExternalFileLinkScheme::File,
            PathBuf::from("path/to/file"),
            None
        )
    )
}

#[test]
fn vimwiki_raw_link() {
    let x = vimwiki_raw_link!("https://example.com");
    assert_eq!(
        x.element,
        RawLink::new(
            URI::try_from("https://example.com").unwrap().into_owned()
        )
    );
}

#[test]
fn vimwiki_transclusion_link() {
    let x = vimwiki_transclusion_link!("{{https://example.com/img.jpg}}");
    assert_eq!(
        x.element,
        TransclusionLink::from(
            URI::try_from("https://example.com/img.jpg")
                .unwrap()
                .into_owned()
        )
    );
}

#[test]
fn vimwiki_wiki_link() {
    let x = vimwiki_wiki_link!("[[link]]");
    assert_eq!(x.element, WikiLink::from(PathBuf::from("link")));
}

#[test]
fn vimwiki_inter_wiki_link() {
    let x = vimwiki_inter_wiki_link!("[[wiki1:link]]");
    assert_eq!(
        x.element,
        InterWikiLink::Indexed(IndexedInterWikiLink::new(
            1,
            WikiLink::from(PathBuf::from("link"))
        ))
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
        x.element,
        List::new(vec![
            LE::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                0,
                ListItemContents::new(vec![LE::from(
                    ListItemContent::InlineContent(
                        InlineElementContainer::new(vec![LE::from(
                            InlineElement::Text(Text::from("some list item"))
                        )])
                    )
                )]),
                ListItemAttributes::default(),
            ),),
            LE::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                1,
                ListItemContents::new(vec![
                    LE::from(ListItemContent::InlineContent(
                        InlineElementContainer::new(vec![LE::from(
                            InlineElement::Text(Text::from(
                                "some other list item"
                            ))
                        )])
                    )),
                    LE::from(ListItemContent::List(
                        TypedBlockElement::from_list(List::new(vec![
                            LE::from(ListItem::new(
                                ListItemType::Ordered(
                                    OrderedListItemType::Number
                                ),
                                ListItemSuffix::Period,
                                0,
                                ListItemContents::new(vec![LE::from(
                                    ListItemContent::InlineContent(
                                        InlineElementContainer::new(vec![
                                            LE::from(InlineElement::Text(
                                                Text::from("sub list item")
                                            ))
                                        ])
                                    )
                                )]),
                                ListItemAttributes::default(),
                            ))
                        ]))
                    ))
                ]),
                ListItemAttributes::default(),
            ))
        ])
    );
}

#[test]
fn vimwiki_list_raw() {
    let x = vimwiki_list_raw!("- some list item");
    assert_eq!(
        x.element,
        List::new(vec![LE::from(ListItem::new(
            ListItemType::Unordered(UnorderedListItemType::Hyphen),
            ListItemSuffix::None,
            0,
            ListItemContents::new(vec![LE::from(
                ListItemContent::InlineContent(InlineElementContainer::new(
                    vec![LE::from(InlineElement::Text(Text::from(
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
    assert_eq!(x.element, CodeInline::new("code".to_string()));
}

#[test]
fn vimwiki_math_inline() {
    let x = vimwiki_math_inline!("$math$");
    assert_eq!(x.element, MathInline::new("math".to_string()));
}

#[test]
fn vimwiki_math_block() {
    let x = vimwiki_math_block! {r#"
    {{$
    math
    }}$
    "#};
    assert_eq!(x.element, MathBlock::new(vec!["math".to_string()], None));
}

#[test]
fn vimwiki_math_block_raw() {
    let x = vimwiki_math_block_raw! {r#"{{$
    math
    }}$"#};
    assert_eq!(
        x.element,
        MathBlock::new(vec!["    math".to_string()], None)
    );
}

#[test]
fn vimwiki_paragraph() {
    let x = vimwiki_paragraph!("some text");
    assert_eq!(
        x.element,
        Paragraph::new(InlineElementContainer::new(vec![LE::from(
            InlineElement::Text(Text::from("some text"))
        )]))
    );
}

#[test]
fn vimwiki_placeholder() {
    assert_eq!(
        vimwiki_placeholder!("%date 2012-03-05").element,
        Placeholder::Date(NaiveDate::from_ymd(2012, 3, 5)),
    );
    assert_eq!(vimwiki_placeholder!("%nohtml").element, Placeholder::NoHtml,);
    assert_eq!(
        vimwiki_placeholder!("%other some text").element,
        Placeholder::Other {
            name: "other".to_string(),
            value: "some text".to_string()
        },
    );
    assert_eq!(
        vimwiki_placeholder!("%template my_template").element,
        Placeholder::Template("my_template".to_string()),
    );
    assert_eq!(
        vimwiki_placeholder!("%title some text").element,
        Placeholder::Title("some text".to_string()),
    );
}

#[test]
fn vimwiki_preformatted_text() {
    let x = vimwiki_preformatted_text! {r#"
    {{{
    some code
    }}}
    "#};
    assert_eq!(
        x.element,
        PreformattedText::new(
            None,
            Default::default(),
            vec!["some code".to_string()]
        )
    );
}

#[test]
fn vimwiki_preformatted_text_raw() {
    let x = vimwiki_preformatted_text_raw! {r#"{{{
    some code
    }}}"#};
    assert_eq!(
        x.element,
        PreformattedText::new(
            None,
            Default::default(),
            vec!["    some code".to_string()]
        )
    );
}

#[test]
fn vimwiki_table() {
    let x = vimwiki_table!("|cell|");
    assert_eq!(
        x.element,
        Table::new(
            vec![LE::from(Row::Content {
                cells: vec![LE::from(Cell::Content(
                    InlineElementContainer::new(vec![LE::from(
                        InlineElement::Text(Text::from("cell"))
                    )])
                ))],
            })],
            false
        )
    );
}

#[test]
fn vimwiki_tags() {
    let x = vimwiki_tags!(":tag:");
    assert_eq!(x.element, Tags::from("tag"));
}

#[test]
fn vimwiki_text() {
    let x = vimwiki_text!("some text");
    assert_eq!(x.element, Text::from("some text"));
}

#[test]
fn vimwiki_decorated_text_bold() {
    let x = vimwiki_decorated_text!("*some text*");
    assert_eq!(
        x.element,
        DecoratedText::Bold(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_bolditalic_1() {
    let x = vimwiki_decorated_text!("*_some text_*");
    assert_eq!(
        x.element,
        DecoratedText::BoldItalic(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_bolditalic_2() {
    let x = vimwiki_decorated_text!("_*some text*_");
    assert_eq!(
        x.element,
        DecoratedText::BoldItalic(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_italic() {
    let x = vimwiki_decorated_text!("_some text_");
    assert_eq!(
        x.element,
        DecoratedText::Italic(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_strikeout() {
    let x = vimwiki_decorated_text!("~~some text~~");
    assert_eq!(
        x.element,
        DecoratedText::Strikeout(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_superscript() {
    let x = vimwiki_decorated_text!("^some text^");
    assert_eq!(
        x.element,
        DecoratedText::Superscript(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_decorated_text_subscript() {
    let x = vimwiki_decorated_text!(",,some text,,");
    assert_eq!(
        x.element,
        DecoratedText::Subscript(vec![LE::from(DecoratedTextContent::from(
            Text::from("some text")
        ))],)
    );
}

#[test]
fn vimwiki_keyword() {
    let x = vimwiki_keyword!("TODO");
    assert_eq!(x.element, Keyword::TODO);
}
