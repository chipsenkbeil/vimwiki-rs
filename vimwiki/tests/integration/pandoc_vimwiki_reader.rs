use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_elements},
};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, Region, LE};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LE<Page> =
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
        vimwiki_header!("=n=")
            .take_with_region(Region::from((24, 1, 24, 4)))
            .into(),
        blank_line().take_with_region(Region::from((25, 1, 25, 1))),
        vimwiki_paragraph!("=== not a header ====")
            .take_with_region(Region::from((26, 1, 26, 22)))
            .into(),
        blank_line().take_with_region(Region::from((27, 1, 27, 1))),
        vimwiki_header_raw!(" == centred header ==")
            .take_with_region(Region::from((28, 1, 28, 22)))
            .into(),
        blank_line().take_with_region(Region::from((29, 1, 29, 1))),
        vimwiki_header!("== header with some `==` in between ==")
            .take_with_region(Region::from((30, 1, 30, 39)))
            .into(),
        vimwiki_header!("== header with some == in between ==")
            .take_with_region(Region::from((31, 1, 31, 37)))
            .into(),
        vimwiki_header!("== header with some ==in between ==")
            .take_with_region(Region::from((32, 1, 32, 36)))
            .into(),
        blank_line().take_with_region(Region::from((33, 1, 33, 1))),
        vimwiki_header!("== emph strong and strikeout ==")
            .take_with_region(Region::from((34, 1, 34, 32)))
            .into(),
        blank_line().take_with_region(Region::from((35, 1, 35, 1))),
        vimwiki_paragraph!("_emph_ *strong*")
            .take_with_region(Region::from((36, 1, 36, 16)))
            .into(),
        blank_line().take_with_region(Region::from((37, 1, 37, 1))),
        vimwiki_paragraph!("*_strong and emph_*")
            .take_with_region(Region::from((38, 1, 38, 20)))
            .into(),
        blank_line().take_with_region(Region::from((39, 1, 39, 1))),
        vimwiki_paragraph!("_*emph and strong*_")
            .take_with_region(Region::from((40, 1, 40, 20)))
            .into(),
        blank_line().take_with_region(Region::from((41, 1, 41, 1))),
        vimwiki_paragraph!("*_emph inside_ strong*")
            .take_with_region(Region::from((42, 1, 42, 23)))
            .into(),
        blank_line().take_with_region(Region::from((43, 1, 43, 1))),
        vimwiki_paragraph!("*strong with _emph_*")
            .take_with_region(Region::from((44, 1, 44, 21)))
            .into(),
        blank_line().take_with_region(Region::from((45, 1, 45, 1))),
        vimwiki_paragraph!("_*strong inside* emph_")
            .take_with_region(Region::from((46, 1, 46, 23)))
            .into(),
        blank_line().take_with_region(Region::from((47, 1, 47, 1))),
        vimwiki_paragraph!("_~~strikeout~~ inside emph_")
            .take_with_region(Region::from((48, 1, 48, 28)))
            .into(),
        blank_line().take_with_region(Region::from((49, 1, 49, 1))),
        vimwiki_paragraph!("~~This is _struck out_ with emph~~")
            .take_with_region(Region::from((50, 1, 50, 35)))
            .into(),
        blank_line().take_with_region(Region::from((51, 1, 51, 1))),
        vimwiki_paragraph! {r#"
            *not
            strong*
        "#}
        .take_with_region(Region::from((52, 1, 53, 8)))
        .into(),
        blank_line().take_with_region(Region::from((54, 1, 54, 1))),
        vimwiki_paragraph!("just two stars: **")
            .take_with_region(Region::from((55, 1, 55, 19)))
            .into(),
        blank_line().take_with_region(Region::from((56, 1, 56, 1))),
        vimwiki_paragraph!("just two underscores: __")
            .take_with_region(Region::from((57, 1, 57, 25)))
            .into(),
        blank_line().take_with_region(Region::from((58, 1, 58, 1))),
        vimwiki_paragraph!("just four ~s: ~~~~")
            .take_with_region(Region::from((59, 1, 59, 19)))
            .into(),
        blank_line().take_with_region(Region::from((60, 1, 60, 1))),
        vimwiki_paragraph!("_not")
            .take_with_region(Region::from((61, 1, 61, 5)))
            .into(),
        blank_line().take_with_region(Region::from((62, 10, 62, 10))),
        vimwiki_paragraph!("emph_")
            .take_with_region(Region::from((63, 1, 63, 6)))
            .into(),
        blank_line().take_with_region(Region::from((64, 1, 64, 1))),
        vimwiki_paragraph!("~~not")
            .take_with_region(Region::from((65, 1, 65, 6)))
            .into(),
        blank_line().take_with_region(Region::from((66, 1, 66, 14))),
        blank_line().take_with_region(Region::from((67, 1, 67, 12))),
        vimwiki_paragraph!("strikeout~~")
            .take_with_region(Region::from((68, 1, 68, 12)))
            .into(),
        blank_line().take_with_region(Region::from((69, 1, 69, 1))),
        vimwiki_header!("== horizontal rule ==")
            .take_with_region(Region::from((70, 1, 70, 22)))
            .into(),
        blank_line().take_with_region(Region::from((71, 1, 71, 1))),
        vimwiki_paragraph!("top")
            .take_with_region(Region::from((72, 1, 72, 4)))
            .into(),
        vimwiki_divider!("----")
            .take_with_region(Region::from((73, 1, 73, 5)))
            .into(),
        vimwiki_paragraph!("middle")
            .take_with_region(Region::from((74, 1, 74, 7)))
            .into(),
        blank_line().take_with_region(Region::from((75, 1, 75, 1))),
        vimwiki_divider!("-------")
            .take_with_region(Region::from((76, 1, 76, 8)))
            .into(),
        blank_line().take_with_region(Region::from((77, 1, 77, 1))),
        vimwiki_paragraph!("not a rule-----")
            .take_with_region(Region::from((78, 1, 78, 16)))
            .into(),
        blank_line().take_with_region(Region::from((79, 1, 79, 1))),
    ];

    compare_page_elements(&page.elements, &expected);
}
