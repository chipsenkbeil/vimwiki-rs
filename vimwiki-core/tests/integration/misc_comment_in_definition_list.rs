use super::fixtures::VimwikiFile;
use std::borrow::Cow;
use vimwiki::*;

#[test]
fn test() {
    let contents = VimwikiFile::MiscCommentInDefinitionList.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    assert_eq!(
        page[0],
        Located::from(BlockElement::from(Paragraph::new(vec![
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(LineComment::from(
                    "term1:: def1"
                )))
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(LineComment::from(
                    "term2:: def2"
                )))
            )]),
            InlineElementContainer::new(vec![Located::from(
                InlineElement::from(Comment::from(MultiLineComment::new(
                    vec![Cow::Borrowed("term3::"), Cow::Borrowed(":: def3")]
                )))
            )]),
        ]))),
    );
}
