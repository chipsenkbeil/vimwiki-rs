use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_elements},
};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, Region, LE};
use vimwiki_macros::*;

/// For testing purposes, moves the located element to the specified line
/// and pushes its column out by one so we cover newlines without needing
/// to include them
fn adjust<T>(le: LE<T>, line: usize) -> LE<T> {
    let mut le = le.take_at_line(line);
    le.region.end.column += 1;
    le
}

#[test]
fn test() {
    let page: LE<Page> =
        RawStr::Vimwiki(&VimwikiFile::PandocVimwikiReader.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        adjust(vimwiki_header!("= _*implemented*_ ="), 1).into(),
        adjust(vimwiki_header!("= header ="), 2).into(),
        blank_line().take_at_line(3),
        adjust(vimwiki_header!("== header level two =="), 4).into(),
        blank_line().take_at_line(5),
        adjust(vimwiki_header!("=== header `level` 3 ==="), 6).into(),
        blank_line().take_at_line(7),
        adjust(vimwiki_header!("==== header ~~level~~ four ===="), 8).into(),
        blank_line().take_at_line(9),
        adjust(vimwiki_header!("===== header _*level* 5_ ====="), 10).into(),
        blank_line().take_at_line(11),
        adjust(vimwiki_header!("====== header level 6 ======"), 12).into(),
        blank_line().take_at_line(13),
        adjust(vimwiki_paragraph!("======= not a header ========"), 14).into(),
        blank_line().take_at_line(15),
        adjust(vimwiki_paragraph!("hi== not a header =="), 16).into(),
        blank_line().take_at_line(17),
        adjust(vimwiki_paragraph!("=== not a header =="), 18).into(),
        blank_line().take_at_line(19),
        adjust(vimwiki_paragraph!("=== not a header ===-"), 20).into(),
        blank_line().take_at_line(21),
        adjust(vimwiki_paragraph!("not a header:"), 22).into(),
        blank_line().take_at_line(23),
        adjust(vimwiki_header!("=n="), 24).into(),
        blank_line().take_at_line(25),
        adjust(vimwiki_paragraph!("=== not a header ===="), 26).into(),
        blank_line().take_at_line(27),
        adjust(vimwiki_header_raw!(" == centred header =="), 28).into(),
        blank_line().take_with_region(Region::from((29, 1, 29, 2))),
        adjust(
            vimwiki_header!("== header with some `==` in between =="),
            30,
        )
        .into(),
        adjust(vimwiki_header!("== header with some == in between =="), 31)
            .into(),
        adjust(vimwiki_header!("== header with some ==in between =="), 32)
            .into(),
        blank_line().take_at_line(33),
        adjust(vimwiki_header!("== emph strong and strikeout =="), 34).into(),
        blank_line().take_at_line(35),
        adjust(vimwiki_paragraph!("_emph_ *strong*"), 36).into(),
        blank_line().take_at_line(37),
        adjust(vimwiki_paragraph!("*_strong and emph_*"), 38).into(),
        blank_line().take_at_line(39),
        adjust(vimwiki_paragraph!("_*emph and strong*_"), 40).into(),
        blank_line().take_at_line(41),
        adjust(vimwiki_paragraph!("*_emph inside_ strong*"), 42).into(),
        blank_line().take_at_line(43),
        adjust(vimwiki_paragraph!("*strong with _emph_*"), 44).into(),
        blank_line().take_at_line(45),
        adjust(vimwiki_paragraph!("_*strong inside* emph_"), 46).into(),
        blank_line().take_at_line(47),
        adjust(vimwiki_paragraph!("_~~strikeout~~ inside emph_"), 48).into(),
        blank_line().take_at_line(49),
        adjust(
            vimwiki_paragraph!("~~This is _struck out_ with emph~~ "),
            50,
        )
        .into(),
        blank_line().take_at_line(51),
        adjust(
            vimwiki_paragraph! {r#"
            *not
            strong*
        "#},
            52,
        )
        .into(),
        blank_line().take_at_line(54),
        adjust(vimwiki_paragraph!("just two stars: **"), 55).into(),
        blank_line().take_at_line(56),
        adjust(vimwiki_paragraph!("just two underscores: __"), 57).into(),
        blank_line().take_at_line(58),
        adjust(vimwiki_paragraph!("just four ~s: ~~~~"), 59).into(),
        blank_line().take_at_line(60),
        adjust(vimwiki_paragraph!("_not"), 61).into(),
        blank_line().take_with_region(Region::from((62, 10, 62, 10))),
        adjust(vimwiki_paragraph!("emph_"), 63).into(),
        blank_line().take_at_line(64),
        adjust(vimwiki_paragraph!("~~not"), 65).into(),
        blank_line().take_with_region(Region::from((66, 1, 66, 14))),
        blank_line().take_with_region(Region::from((67, 1, 67, 12))),
        adjust(vimwiki_paragraph!("strikeout~~"), 68).into(),
        blank_line().take_at_line(69),
        adjust(vimwiki_header!("== horizontal rule =="), 70).into(),
        blank_line().take_at_line(71),
        adjust(vimwiki_paragraph!("top"), 72).into(),
        adjust(vimwiki_divider!("----"), 73).into(),
        adjust(vimwiki_paragraph!("middle"), 74).into(),
        blank_line().take_at_line(75),
        adjust(vimwiki_divider!("-------"), 76).into(),
        blank_line().take_at_line(77),
        adjust(vimwiki_paragraph!("not a rule-----"), 78).into(),
        blank_line().take_at_line(79),
        adjust(
            vimwiki_paragraph! {r#"
                not a rule (trailing spaces):
                ----- 
            "#},
            80,
        )
        .into(),
    ];

    // TODO: Validate comments
    compare_page_elements(&page.elements, &expected);
}
