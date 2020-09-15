use std::{convert::TryFrom, path::PathBuf};
use vimwiki::{
    components::*,
    vendor::{chrono::NaiveDate, uriparse::URI},
    LC,
};
use vimwiki_macros::*;

#[test]
fn vimwiki_page() {
    let x = vimwiki_page!("some text");
    assert_eq!(
        x.component,
        Page::new(
            vec![LC::from(BlockComponent::Paragraph(Paragraph::new(
                InlineComponentContainer::new(vec![LC::from(
                    InlineComponent::Text("some text".to_string())
                )])
            )))],
            vec![]
        )
    );
}

#[test]
fn vimwiki_block_component() {
    let x = vimwiki_block_component!("some text");
    assert_eq!(
        x.component,
        BlockComponent::Paragraph(Paragraph::new(
            InlineComponentContainer::new(vec![LC::from(
                InlineComponent::Text("some text".to_string())
            )])
        ))
    );
}

#[test]
fn vimwiki_inline_component_container() {
    let x = vimwiki_inline_component_container!("some text");
    assert_eq!(
        x.component,
        InlineComponentContainer::new(vec![LC::from(InlineComponent::Text(
            "some text".to_string()
        ))])
    );
}

#[test]
fn vimwiki_inline_component() {
    let x = vimwiki_inline_component!("some text");
    assert_eq!(x.component, InlineComponent::Text("some text".to_string()));
}

#[test]
fn vimwiki_blockquote() {
    let x = vimwiki_blockquote!("> some text");
    assert_eq!(x.component, Blockquote::new(vec!["some text".to_string()]));
}

#[test]
fn vimwiki_comment() {
    let x = vimwiki_comment!("%% some comment");
    assert_eq!(
        x.component,
        Comment::from(LineComment(" some comment".to_string()))
    );
}

#[test]
fn vimwiki_line_comment() {
    let x = vimwiki_line_comment!("%% some comment");
    assert_eq!(x.component, LineComment(" some comment".to_string()));
}

#[test]
fn vimwiki_multi_line_comment() {
    let x = vimwiki_multi_line_comment!("%%+ some comment +%%");
    assert_eq!(
        x.component,
        MultiLineComment(vec![" some comment ".to_string()])
    );
}

#[test]
fn vimwiki_definition_list() {
    let x = vimwiki_definition_list!("term:: definition");
    assert_eq!(
        x.component,
        DefinitionList::from(vec![TermAndDefinitions::new(
            LC::from("term".to_string()),
            vec![LC::from("definition".to_string())],
        )])
    );
}

#[test]
fn vimwiki_divider() {
    let x = vimwiki_divider!("----");
    assert_eq!(x.component, Divider);
}

#[test]
fn vimwiki_header() {
    let x = vimwiki_header!("= header =");
    assert_eq!(x.component, Header::new(1, "header".to_string(), false));
}

#[test]
fn vimwiki_link() {
    let x = vimwiki_link!("[[link]]");
    assert_eq!(
        x.component,
        Link::Wiki(WikiLink::from(PathBuf::from("link")))
    );
}

#[test]
fn vimwiki_diary_link() {
    let x = vimwiki_diary_link!("[[diary:2012-03-05]]");
    assert_eq!(
        x.component,
        DiaryLink::from(NaiveDate::from_ymd(2012, 3, 5))
    );
}

#[test]
fn vimwiki_external_file_link() {
    let x = vimwiki_external_file_link!("[[file:path/to/file]]");
    assert_eq!(
        x.component,
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
        x.component,
        RawLink::new(
            URI::try_from("https://example.com").unwrap().into_owned()
        )
    );
}

#[test]
fn vimwiki_transclusion_link() {
    let x = vimwiki_transclusion_link!("{{https://example.com/img.jpg}}");
    assert_eq!(
        x.component,
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
    assert_eq!(x.component, WikiLink::from(PathBuf::from("link")));
}

#[test]
fn vimwiki_inter_wiki_link() {
    let x = vimwiki_inter_wiki_link!("[[wiki1:link]]");
    assert_eq!(
        x.component,
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
        x.component,
        List::new(vec![
            LC::from(EnhancedListItem::new(
                ListItem::new(
                    ListItemType::Unordered(UnorderedListItemType::Hyphen),
                    ListItemSuffix::None,
                    0,
                    ListItemContents::new(vec![LC::from(
                        ListItemContent::InlineContent(
                            InlineComponentContainer::new(vec![LC::from(
                                InlineComponent::Text(
                                    "some list item".to_string()
                                )
                            )])
                        )
                    )]),
                ),
                Default::default(),
            )),
            LC::from(EnhancedListItem::new(
                ListItem::new(
                    ListItemType::Unordered(UnorderedListItemType::Hyphen),
                    ListItemSuffix::None,
                    1,
                    ListItemContents::new(vec![
                        LC::from(ListItemContent::InlineContent(
                            InlineComponentContainer::new(vec![LC::from(
                                InlineComponent::Text(
                                    "some other list item".to_string()
                                )
                            )])
                        )),
                        LC::from(ListItemContent::List(List::new(vec![
                            LC::from(EnhancedListItem::new(
                                ListItem::new(
                                    ListItemType::Ordered(
                                        OrderedListItemType::Number
                                    ),
                                    ListItemSuffix::Period,
                                    0,
                                    ListItemContents::new(vec![LC::from(
                                        ListItemContent::InlineContent(
                                            InlineComponentContainer::new(
                                                vec![LC::from(
                                                    InlineComponent::Text(
                                                        "sub list item"
                                                            .to_string()
                                                    )
                                                )]
                                            )
                                        )
                                    )]),
                                ),
                                Default::default(),
                            )),
                        ],)))
                    ]),
                ),
                Default::default(),
            ))
        ])
    );
}

#[test]
fn vimwiki_list_raw() {
    let x = vimwiki_list_raw!("- some list item");
    assert_eq!(
        x.component,
        List::new(vec![LC::from(EnhancedListItem::new(
            ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                0,
                ListItemContents::new(vec![LC::from(
                    ListItemContent::InlineContent(
                        InlineComponentContainer::new(vec![LC::from(
                            InlineComponent::Text("some list item".to_string())
                        )])
                    )
                )]),
            ),
            Default::default(),
        ))])
    );
}

#[test]
fn vimwiki_math_inline() {
    let x = vimwiki_math_inline!("$math$");
    assert_eq!(x.component, MathInline::new("math".to_string()));
}

#[test]
fn vimwiki_math_block() {
    let x = vimwiki_math_block! {r#"
    {{$
    math
    }}$
    "#};
    assert_eq!(x.component, MathBlock::new(vec!["math".to_string()], None));
}

#[test]
fn vimwiki_math_block_raw() {
    let x = vimwiki_math_block_raw! {r#"{{$
    math
    }}$"#};
    assert_eq!(
        x.component,
        MathBlock::new(vec!["    math".to_string()], None)
    );
}

#[test]
fn vimwiki_paragraph() {
    let x = vimwiki_paragraph!("some text");
    assert_eq!(
        x.component,
        Paragraph::new(InlineComponentContainer::new(vec![LC::from(
            InlineComponent::Text("some text".to_string())
        )]))
    );
}

#[test]
fn vimwiki_placeholder() {
    assert_eq!(
        vimwiki_placeholder!("%date 2012-03-05").component,
        Placeholder::Date(NaiveDate::from_ymd(2012, 3, 5)),
    );
    assert_eq!(
        vimwiki_placeholder!("%nohtml").component,
        Placeholder::NoHtml,
    );
    assert_eq!(
        vimwiki_placeholder!("%other some text").component,
        Placeholder::Other {
            name: "other".to_string(),
            value: "some text".to_string()
        },
    );
    assert_eq!(
        vimwiki_placeholder!("%template my_template").component,
        Placeholder::Template("my_template".to_string()),
    );
    assert_eq!(
        vimwiki_placeholder!("%title some text").component,
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
        x.component,
        PreformattedText::new(
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
        x.component,
        PreformattedText::new(
            Default::default(),
            vec!["    some code".to_string()]
        )
    );
}

#[test]
fn vimwiki_table() {
    let x = vimwiki_table!("|cell|");
    assert_eq!(
        x.component,
        Table::new(
            vec![LC::from(Row::Content {
                cells: vec![LC::from(Cell::Content(
                    InlineComponentContainer::new(vec![LC::from(
                        InlineComponent::Text("cell".to_string())
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
    assert_eq!(x.component, Tags::from("tag"));
}

#[test]
fn vimwiki_string() {
    let x = vimwiki_string!("some text");
    assert_eq!(x.component, "some text".to_string());
}

#[test]
fn vimwiki_decorated_text() {
    let x = vimwiki_decorated_text!("*some text*");
    assert_eq!(
        x.component,
        DecoratedText::new(
            vec![LC::from(DecoratedTextContent::Text(
                "some text".to_string()
            ))],
            Decoration::Bold
        )
    );
}

#[test]
fn vimwiki_keyword() {
    let x = vimwiki_keyword!("TODO");
    assert_eq!(x.component, Keyword::TODO);
}
