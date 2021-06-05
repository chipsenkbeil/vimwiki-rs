use crate::fixtures::VimwikiFile;
use std::convert::TryFrom;
use uriparse::URIReference;
use vimwiki_core::*;

/// https://github.com/chipsenkbeil/vimwiki-rs/issues/119
#[test]
fn issue_119() {
    let contents = VimwikiFile::Issue119.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    // some words in front: https://example.com/
    assert_eq!(
        page[0],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![
                Located::from(InlineElement::from(Text::from(
                    "some words in front: "
                ))),
                Located::from(InlineElement::from(Link::new_raw_link(
                    URIReference::try_from("https://example.com/").unwrap()
                )))
            ]),
        ])))
    );
}

/// https://github.com/chipsenkbeil/vimwiki-rs/issues/120
#[test]
fn issue_120() {
    let contents = VimwikiFile::Issue120.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    // == not tags ==
    assert_eq!(
        page[0],
        Located::from(BlockElement::from(Header::new(
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("not tags"))
            )]),
            2,
            false,
        )))
    );

    // 2021-05-30 12:35:03.913609534-07:00
    assert_eq!(
        page[1],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from(
                    "2021-05-30 12:35:03.913609534-07:00"
                ))
            )]),
        ])))
    );

    // 2021-05-30 15:41:15-06:00
    assert_eq!(
        page[2],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("2021-05-30 15:41:15-06:00"))
            )]),
        ])))
    );

    // 15:41:15
    assert_eq!(
        page[3],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("15:41:15"))
            )]),
        ])))
    );

    // foo:bar:baz
    assert_eq!(
        page[4],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("foo:bar:baz"))
            )]),
        ])))
    );
}

/// https://github.com/chipsenkbeil/vimwiki-rs/issues/122
#[test]
fn issue_122() {
    let contents = VimwikiFile::Issue122.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    // == comments ==
    assert_eq!(
        page[0],
        Located::from(BlockElement::from(Header::new(
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Text::from("comments"))
            )]),
            2,
            false,
        )))
    );

    // %% this is a comment
    assert_eq!(
        page[1],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(LineComment::from(
                    " this is a comment"
                )))
            )]),
        ])))
    );

    // %% this is a comment with embedded:colons:in
    assert_eq!(
        page[2],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(LineComment::from(
                    " this is a comment with embedded:colons:in"
                )))
            )]),
        ])))
    );

    // %% mark :: 2017-01-04T15:27:39-0700
    assert_eq!(
        page[3],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(LineComment::from(
                    " mark :: 2017-01-04T15:27:39-0700"
                )))
            )]),
        ])))
    );
}
