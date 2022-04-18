use crate::parser::fixtures::VimwikiFile;
use std::borrow::Cow;
use vimwiki_core::*;

#[test]
fn test() {
    let contents = VimwikiFile::MiscWindowsSupport.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();
    assert_eq!(
        page[0],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("this is a paragraph"))
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("with carriage returns"))
            )]),
        ]))),
    );

    assert_eq!(
        page[1],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(MultiLineComment::new(
                    vec![
                        Cow::Borrowed("this is a comment"),
                        Cow::Borrowed("with carriage returns"),
                    ]
                )))
            )]),
        ]))),
    );

    assert_eq!(
        page[2],
        Located::from(BlockElement::from(CodeBlock::from_lines(vec![
            "this is a code block",
            "with carriage returns",
        ]))),
    );

    assert_eq!(
        page[3],
        Located::from(BlockElement::from(MathBlock::from_lines(vec![
            "this is a math block",
            "with carriage returns",
        ]))),
    );

    assert_eq!(
        page[4],
        Located::from(BlockElement::from(List::new(vec![
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                0,
                ListItemContents::new(vec![
                    Located::from(BlockElement::from(Paragraph::new(vec![
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::from(Text::from("this is a list"))
                        )])
                    ]))),
                    Located::from(BlockElement::List(List::new(vec![
                        Located::from(ListItem::new(
                            ListItemType::Unordered(
                                UnorderedListItemType::Hyphen
                            ),
                            ListItemSuffix::None,
                            0,
                            ListItemContents::new(vec![Located::from(
                                BlockElement::from(Paragraph::new(vec![
                                    InlineElementContainer::new(vec![
                                        Located::from(InlineElement::from(
                                            Text::from("and a sublist")
                                        ))
                                    ])
                                ]))
                            )]),
                            ListItemAttributes::default(),
                        ))
                    ]))),
                    Located::from(BlockElement::from(Paragraph::new(vec![
                        InlineElementContainer::new(vec![Located::from(
                            InlineElement::from(Text::from(
                                "with carriage returns"
                            ))
                        )])
                    ])))
                ]),
                ListItemAttributes::default(),
            )),
            Located::from(ListItem::new(
                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                ListItemSuffix::None,
                1,
                ListItemContents::new(vec![Located::from(BlockElement::from(
                    Paragraph::new(vec![InlineElementContainer::new(vec![
                        Located::from(InlineElement::from(Text::from(
                            "and multiple items"
                        )))
                    ])])
                ))]),
                ListItemAttributes::default(),
            ))
        ]))),
    );

    assert_eq!(
        page[5],
        Located::from(BlockElement::from(DefinitionList::new(vec![
            Located::from(TermAndDefinitions::new(
                Located::from(Term::from("term1")),
                Located::from(DefinitionBundle::new(vec![Located::from(
                    Definition::from("with carriage returns")
                )]))
            )),
            Located::from(TermAndDefinitions::new(
                Located::from(Term::from("term2")),
                Located::from(DefinitionBundle::new(vec![Located::from(
                    Definition::from("with carriage returns")
                )]))
            ))
        ]))),
    );

    assert_eq!(
        page[6],
        Located::from(BlockElement::from(Header::new(
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("header with carriage returns"))
            )]),
            3,
            false,
        ))),
    );

    assert_eq!(page[7], Located::from(BlockElement::from(Divider::new())),);

    assert_eq!(
        page[8],
        Located::from(BlockElement::from(Placeholder::Title(Cow::Borrowed(
            "some title"
        )))),
    );

    assert_eq!(
        page[9],
        Located::from(BlockElement::from(Table::new(
            vec![
                (
                    CellPos::new(0, 0),
                    Located::from(Cell::Content(InlineElementContainer::new(
                        vec![Located::from(InlineElement::from(Text::from(
                            " table                 "
                        )))]
                    )))
                ),
                (
                    CellPos::new(1, 0),
                    Located::from(Cell::Align(ColumnAlign::default()))
                ),
                (
                    CellPos::new(2, 0),
                    Located::from(Cell::Content(InlineElementContainer::new(
                        vec![Located::from(InlineElement::from(Text::from(
                            " with carriage returns "
                        )))]
                    )))
                )
            ],
            false
        ))),
    );

    assert_eq!(
        page[10],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(CodeInline::from(
                    "inline code with carriage returns"
                ))
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(MathInline::from(
                    "inline math with carriage returns"
                ))
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(
                    Link::try_new_wiki_link(
                        "wiki%20link%20with%20carriage%20returns",
                        None,
                    )
                    .unwrap()
                )
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(
                    Link::try_new_transclusion_link(
                        "transclusion%20link%20with%20carriage%20returns",
                        None,
                        None,
                    )
                    .unwrap()
                )
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(
                    Link::try_new_raw_link(
                        "https://raw-link-with-carriage-returns.example.com/",
                    )
                    .unwrap()
                )
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(
                    vec!["tag", "with", "carriage", "returns"]
                        .into_iter()
                        .collect::<Tags>()
                )
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("ending text"))
            )]),
        ]))),
    );
}
