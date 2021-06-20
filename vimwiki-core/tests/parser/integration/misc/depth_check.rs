use crate::parser::fixtures::VimwikiFile;
use similar_asserts::Diff;
use std::convert::TryFrom;
use vimwiki_core::{vendor::uriparse::URIReference, *};

#[test]
fn test() {
    // NOTE: On windows, loading the file into a string will yield \r\n for
    //       the line terminations regardless of what the file actually
    //       contains; so, we need to remove \r to handle this test
    let contents: String = VimwikiFile::MiscDepthCheck
        .load()
        .unwrap()
        .chars()
        .filter(|c| *c != '\r')
        .collect();

    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    // NOTE: Regions are based on \n only and not \r\n
    let expected_page = Page::new(vec![
        Located::new(
            BlockElement::from(Header::new(
                InlineElementContainer::new(vec![Located::new(
                    InlineElement::from(Text::from("Header 1")),
                    Region::new_at_depth(2, 8, 1),
                )]),
                1,
                false,
            )),
            Region::new_at_depth(0, 13, 0),
        ),
        Located::new(
            BlockElement::from(Paragraph::new(vec![
                InlineElementContainer::new(vec![
                    Located::new(
                        InlineElement::from(Text::from(
                            "Paragraph with text, ",
                        )),
                        Region::new_at_depth(14, 21, 1),
                    ),
                    Located::new(
                        InlineElement::from(DecoratedText::Bold(vec![
                            Located::new(
                                DecoratedTextContent::from(Text::from("bold")),
                                Region::new_at_depth(36, 4, 2),
                            ),
                        ])),
                        Region::new_at_depth(35, 6, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", ")),
                        Region::new_at_depth(41, 2, 1),
                    ),
                    Located::new(
                        InlineElement::from(DecoratedText::Italic(vec![
                            Located::new(
                                DecoratedTextContent::from(Text::from(
                                    "italic",
                                )),
                                Region::new_at_depth(44, 6, 2),
                            ),
                        ])),
                        Region::new_at_depth(43, 8, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", ")),
                        Region::new_at_depth(51, 2, 1),
                    ),
                    Located::new(
                        InlineElement::from(DecoratedText::Strikeout(vec![
                            Located::new(
                                DecoratedTextContent::from(Text::from(
                                    "strikeout",
                                )),
                                Region::new_at_depth(55, 9, 2),
                            ),
                        ])),
                        Region::new_at_depth(53, 13, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", ")),
                        Region::new_at_depth(66, 2, 1),
                    ),
                    Located::new(
                        InlineElement::from(CodeInline::from("code")),
                        Region::new_at_depth(68, 6, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", ")),
                        Region::new_at_depth(74, 2, 1),
                    ),
                    Located::new(
                        InlineElement::from(MathInline::from("math")),
                        Region::new_at_depth(76, 6, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(",")),
                        Region::new_at_depth(82, 1, 1),
                    ),
                ]),
                InlineElementContainer::new(vec![
                    Located::new(
                        InlineElement::from(DecoratedText::Superscript(vec![
                            Located::new(
                                DecoratedTextContent::from(Text::from(
                                    "superscript",
                                )),
                                Region::new_at_depth(85, 11, 2),
                            ),
                        ])),
                        Region::new_at_depth(84, 13, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", ")),
                        Region::new_at_depth(97, 2, 1),
                    ),
                    Located::new(
                        InlineElement::from(DecoratedText::Subscript(vec![
                            Located::new(
                                DecoratedTextContent::from(Text::from(
                                    "subscript",
                                )),
                                Region::new_at_depth(101, 9, 2),
                            ),
                        ])),
                        Region::new_at_depth(99, 13, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(", and ")),
                        Region::new_at_depth(112, 6, 1),
                    ),
                    Located::new(
                        InlineElement::from(Link::new_wiki_link(URIReference::try_from("links").unwrap(), None)),
                        Region::new_at_depth(118, 9, 1),
                    ),
                    Located::new(
                        InlineElement::from(Text::from(".")),
                        Region::new_at_depth(127, 1, 1),
                    ),
                ],
            )
            ])),
            Region::new_at_depth(14, 115, 0),
        ),
        Located::new(
            BlockElement::from(List::new(vec![
                Located::new(
                    ListItem::new(
                        ListItemType::Unordered(UnorderedListItemType::Hyphen),
                        ListItemSuffix::None,
                        0,
                        ListItemContents::new(vec![
                            Located::new(
                                BlockElement::from(Paragraph::new(vec![
                                    InlineElementContainer::new(vec![
                                        Located::new(
                                            InlineElement::Text(Text::from(
                                                "List of items",
                                            )),
                                            Region::new_at_depth(132, 13, 3),
                                        ),
                                    ]),
                                ])),
                                Region::new_at_depth(132, 13, 2),
                            ),
                            Located::new(
                                BlockElement::List(List::new(vec![
                                    Located::new(
                                        ListItem::new(
                                            ListItemType::Unordered(
                                                UnorderedListItemType::Hyphen,
                                            ),
                                            ListItemSuffix::None,
                                            0,
                                            ListItemContents::new(vec![
                                                Located::new(
                                                    BlockElement::from(Paragraph::new(vec![
                                                        InlineElementContainer::new(vec![
                                                            Located::new(
                                                                InlineElement::Text(Text::from("Containing a sublist")),
                                                                Region::new_at_depth(152, 20, 5)
                                                            ),
                                                        ])
                                                    ])),
                                                    Region::new_at_depth(152, 20, 4),
                                                ),
                                                Located::new(
                                                    BlockElement::List(List::new(vec![
                                                        Located::new(
                                                            ListItem::new(
                                                                ListItemType::Unordered(UnorderedListItemType::Hyphen),
                                                                ListItemSuffix::None,
                                                                0,
                                                                ListItemContents::new(vec![
                                                                    Located::new(
                                                                        BlockElement::from(Paragraph::new(vec![
                                                                            InlineElementContainer::new(vec![
                                                                                Located::new(
                                                                                    InlineElement::Text(Text::from("With another sublist")),
                                                                                    Region::new_at_depth(183, 20, 7),
                                                                                )
                                                                            ])
                                                                        ])),
                                                                        Region::new_at_depth(183, 20, 6)
                                                                    ),
                                                                    Located::new(
                                                                        BlockElement::List(List::new(vec![
                                                                            Located::new(
                                                                                ListItem::new(
                                                                                    ListItemType::Unordered(UnorderedListItemType::Hyphen),
                                                                                    ListItemSuffix::None,
                                                                                    0,
                                                                                    ListItemContents::new(vec![
                                                                                        Located::new(
                                                                                            BlockElement::from(Paragraph::new(vec![
                                                                                                InlineElementContainer::new(vec![
                                                                                                    Located::new(
                                                                                                        InlineElement::Text(Text::from("And an additional sublist")),
                                                                                                        Region::new_at_depth(218, 25, 9),
                                                                                                    )
                                                                                                ])
                                                                                            ])),
                                                                                            Region::new_at_depth(218, 25, 8),
                                                                                            ),
                                                                                    ]),
                                                                                    ListItemAttributes::default(),
                                                                                ),
                                                                                Region::new_at_depth(216, 28, 7),
                                                                            )
                                                                        ])),
                                                                        Region::new_at_depth(204, 40 , 6),
                                                                    ),
                                                                    Located::new(
                                                                        BlockElement::from(Paragraph::new(vec![
                                                                            InlineElementContainer::new(vec![
                                                                                Located::new(
                                                                                    InlineElement::Text(Text::from("with content from the a sublist")),
                                                                                    Region::new_at_depth(254, 31, 7),
                                                                                )
                                                                            ])
                                                                        ])),
                                                                        Region::new_at_depth(254, 31, 6)
                                                                    ),
                                                                ]),
                                                                ListItemAttributes::default(),
                                                            ),
                                                            Region::new_at_depth(181, 105, 5),
                                                        ),
                                                    ])),
                                                    Region::new_at_depth(173, 113, 4),
                                                ),
                                                Located::new(
                                                    BlockElement::from(Paragraph::new(vec![
                                                        InlineElementContainer::new(vec![
                                                            Located::new(
                                                                InlineElement::Text(Text::from("and content after that sublist")),
                                                                Region::new_at_depth(292, 30, 5)
                                                            )
                                                        ])
                                                    ])),
                                                    Region::new_at_depth(292, 30, 4),
                                                ),
                                            ]),
                                            ListItemAttributes::default(),
                                        ),
                                        Region::new_at_depth(150, 173, 3),
                                    ),
                                ])),
                                Region::new_at_depth(146, 177, 2),
                            ),
                            Located::new(
                                BlockElement::from(Paragraph::new(vec![
                                    InlineElementContainer::new(vec![
                                        Located::new(
                                            InlineElement::Text(Text::from("and ")),
                                            Region::new_at_depth(325, 4, 3)
                                        ),
                                        Located::new(
                                            InlineElement::DecoratedText(DecoratedText::Bold(vec![
                                                Located::new(
                                                    DecoratedTextContent::Text(Text::from("bold")),
                                                    Region::new_at_depth(330, 4, 4),
                                                ),
                                            ])),
                                            Region::new_at_depth(329, 6, 3),
                                        ),
                                        Located::new(
                                            InlineElement::Text(Text::from(" content after that sublist")),
                                            Region::new_at_depth(335, 27, 3)
                                        )
                                    ])
                                ])),
                                Region::new_at_depth(325, 37, 2),
                            ),
                        ]),
                        ListItemAttributes::default(),
                    ),
                    Region::new_at_depth(130, 233, 1),
                ),
                Located::new(
                    ListItem::new(
                        ListItemType::Unordered(UnorderedListItemType::Hyphen),
                        ListItemSuffix::None,
                        1,
                        ListItemContents::new(vec![Located::new(
                            BlockElement::from(Paragraph::new(vec![
                                InlineElementContainer::new(vec![
                                    Located::new(
                                        InlineElement::from(Text::from(
                                            "With another item after that",
                                        )),
                                        Region::new_at_depth(365, 28, 3),
                                    ),
                                ]),
                            ])),
                            Region::new_at_depth(365, 28, 2),
                        )]),
                        ListItemAttributes::default(),
                    ),
                    Region::new_at_depth(363, 31, 1),
                ),
            ])),
            Region::new_at_depth(130, 264, 0),
        ),
    ]);

    assert!(
        page.strict_eq(&expected_page),
        "Pages not equal! {}",
        Diff::from_debug(&page, &expected_page, "Actual Page", "Expected Page"),
    );
}
