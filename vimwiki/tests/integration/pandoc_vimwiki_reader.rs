use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_components},
};
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, Region, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::PandocVimwikiReader.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        vimwiki_header!("= _*implemented*_ =")
            .take_with_region(Region::from((1, 1, 1, 20)))
            .into(),
        vimwiki_header!("= header =")
            .take_with_region(Region::from((2, 1, 2, 11)))
            .into(),
        blank_line().take_with_region(Region::from((3, 1, 3, 1))),
        vimwiki_header!("== header level two ==")
            .take_with_region(Region::from((4, 1, 4, 23)))
            .into(),
        blank_line().take_with_region(Region::from((5, 1, 5, 1))),
        vimwiki_header!("=== header `level` 3 ===")
            .take_with_region(Region::from((6, 1, 6, 25)))
            .into(),
        blank_line().take_with_region(Region::from((7, 1, 7, 1))),
        vimwiki_header!("==== header ~~level~~ four ====")
            .take_with_region(Region::from((8, 1, 8, 32)))
            .into(),
        blank_line().take_with_region(Region::from((9, 1, 9, 1))),
        vimwiki_header!("===== header _*level* 5_ =====")
            .take_with_region(Region::from((10, 1, 10, 31)))
            .into(),
        blank_line().take_with_region(Region::from((11, 1, 11, 1))),
        vimwiki_header!("====== header level 6 ======")
            .take_with_region(Region::from((12, 1, 12, 29)))
            .into(),
        blank_line().take_with_region(Region::from((13, 1, 13, 1))),
        vimwiki_paragraph!("======= not a header ========")
            .take_with_region(Region::from((14, 1, 14, 30)))
            .into(),
        blank_line().take_with_region(Region::from((15, 1, 15, 1))),
        vimwiki_paragraph!("hi== not a header ==")
            .take_with_region(Region::from((16, 1, 16, 21)))
            .into(),
        blank_line().take_with_region(Region::from((17, 1, 17, 1))),
        vimwiki_paragraph!("=== not a header ==")
            .take_with_region(Region::from((18, 1, 18, 20)))
            .into(),
        blank_line().take_with_region(Region::from((19, 1, 19, 1))),
        vimwiki_paragraph!("=== not a header ===-")
            .take_with_region(Region::from((20, 1, 20, 22)))
            .into(),
        blank_line().take_with_region(Region::from((21, 1, 21, 1))),
        vimwiki_paragraph!("not a header:")
            .take_with_region(Region::from((22, 1, 22, 14)))
            .into(),
        blank_line().take_with_region(Region::from((23, 1, 23, 1))),
        vimwiki_paragraph!("=n=")
            .take_with_region(Region::from((24, 1, 24, 4)))
            .into(),
        blank_line().take_with_region(Region::from((25, 1, 25, 1))),
        vimwiki_paragraph!("=== not a header ====")
            .take_with_region(Region::from((26, 1, 26, 22)))
            .into(),
        blank_line().take_with_region(Region::from((27, 1, 27, 1))),
        vimwiki_header!(" == centred header ==")
            .take_with_region(Region::from((28, 1, 28, 22)))
            .into(),
        blank_line().take_with_region(Region::from((29, 1, 29, 1))),
        vimwiki_header!("== header with some `==` in between ==")
            .take_with_region(Region::from((30, 1, 30, 39)))
            .into(),
        vimwiki_header!("== header with some == in between ==")
            .take_with_region(Region::from((31, 1, 31, 39)))
            .into(),
        vimwiki_header!("== header with some ==in between ==")
            .take_with_region(Region::from((32, 1, 32, 38)))
            .into(),
        blank_line().take_with_region(Region::from((33, 1, 33, 1))),
        vimwiki_header!("== emph strong and strikeout ==")
            .take_with_region(Region::from((34, 1, 34, 32)))
            .into(),
    ];

    compare_page_components(&page.components, &expected);
}
