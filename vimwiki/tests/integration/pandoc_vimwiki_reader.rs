use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_elements},
};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, Region, LE};
use vimwiki_macros::*;

/// For testing purposes:
///
/// 1. Converts the provided input to an LE<BlockElement>
/// 2. Moves the located element to the specified line
/// 3. Pushes its column out by one so we cover newlines without needing to include them
fn adjust(le: impl Into<LE<BlockElement>>, line: usize) -> LE<BlockElement> {
    let mut le = le.into().take_at_line(line);
    le.region.end.column += 1;
    le
}

fn blank_at_line(line: usize) -> LE<BlockElement> {
    blank_line().take_at_line(line)
}

#[test]
fn test() {
    let page: LE<Page> =
        RawStr::Vimwiki(&VimwikiFile::PandocVimwikiReader.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        adjust(vimwiki_header!("= _*implemented*_ ="), 1),
        adjust(vimwiki_header!("= header ="), 2),
        blank_at_line(3),
        adjust(vimwiki_header!("== header level two =="), 4),
        blank_at_line(5),
        adjust(vimwiki_header!("=== header `level` 3 ==="), 6),
        blank_at_line(7),
        adjust(vimwiki_header!("==== header ~~level~~ four ===="), 8),
        blank_at_line(9),
        adjust(vimwiki_header!("===== header _*level* 5_ ====="), 10),
        blank_at_line(11),
        adjust(vimwiki_header!("====== header level 6 ======"), 12),
        blank_at_line(13),
        adjust(vimwiki_paragraph!("======= not a header ========"), 14),
        blank_at_line(15),
        adjust(vimwiki_paragraph!("hi== not a header =="), 16),
        blank_at_line(17),
        adjust(vimwiki_paragraph!("=== not a header =="), 18),
        blank_at_line(19),
        adjust(vimwiki_paragraph!("=== not a header ===-"), 20),
        blank_at_line(21),
        adjust(vimwiki_paragraph!("not a header:"), 22),
        blank_at_line(23),
        adjust(vimwiki_header!("=n="), 24),
        blank_at_line(25),
        adjust(vimwiki_paragraph!("=== not a header ===="), 26),
        blank_at_line(27),
        adjust(vimwiki_header_raw!(" == centred header =="), 28),
        blank_line().take_with_region(Region::from((29, 1, 29, 2))),
        adjust(
            vimwiki_header!("== header with some `==` in between =="),
            30,
        ),
        adjust(vimwiki_header!("== header with some == in between =="), 31),
        adjust(vimwiki_header!("== header with some ==in between =="), 32),
        blank_at_line(33),
        adjust(vimwiki_header!("== emph strong and strikeout =="), 34),
        blank_at_line(35),
        adjust(vimwiki_paragraph!("_emph_ *strong*"), 36),
        blank_at_line(37),
        adjust(vimwiki_paragraph!("*_strong and emph_*"), 38),
        blank_at_line(39),
        adjust(vimwiki_paragraph!("_*emph and strong*_"), 40),
        blank_at_line(41),
        adjust(vimwiki_paragraph!("*_emph inside_ strong*"), 42),
        blank_at_line(43),
        adjust(vimwiki_paragraph!("*strong with _emph_*"), 44),
        blank_at_line(45),
        adjust(vimwiki_paragraph!("_*strong inside* emph_"), 46),
        blank_at_line(47),
        adjust(vimwiki_paragraph!("_~~strikeout~~ inside emph_"), 48),
        blank_at_line(49),
        adjust(
            vimwiki_paragraph!("~~This is _struck out_ with emph~~ "),
            50,
        ),
        blank_at_line(51),
        adjust(
            vimwiki_paragraph! {r#"
            *not
            strong*
        "#},
            52,
        ),
        blank_at_line(54),
        adjust(vimwiki_paragraph!("just two stars: **"), 55),
        blank_at_line(56),
        adjust(vimwiki_paragraph!("just two underscores: __"), 57),
        blank_at_line(58),
        adjust(vimwiki_paragraph!("just four ~s: ~~~~"), 59),
        blank_at_line(60),
        adjust(vimwiki_paragraph!("_not"), 61),
        blank_line().take_with_region(Region::from((62, 10, 62, 10))),
        adjust(vimwiki_paragraph!("emph_"), 63),
        blank_at_line(64),
        adjust(vimwiki_paragraph!("~~not"), 65),
        blank_line().take_with_region(Region::from((66, 1, 66, 14))),
        blank_line().take_with_region(Region::from((67, 1, 67, 12))),
        adjust(vimwiki_paragraph!("strikeout~~"), 68),
        blank_at_line(69),
        adjust(vimwiki_header!("== horizontal rule =="), 70),
        blank_at_line(71),
        adjust(vimwiki_paragraph!("top"), 72),
        adjust(vimwiki_divider!("----"), 73),
        adjust(vimwiki_paragraph!("middle"), 74),
        blank_at_line(75),
        adjust(vimwiki_divider!("-------"), 76),
        blank_at_line(77),
        adjust(vimwiki_paragraph!("not a rule-----"), 78),
        blank_at_line(79),
        adjust(
            vimwiki_paragraph! {r#"
                not a rule (trailing spaces):
                ----- 
            "#},
            80,
        ),
        blank_at_line(82),
        adjust(
            vimwiki_paragraph! {r#"
                not a rule (leading spaces):
                  ----
            "#},
            83,
        ),
        blank_at_line(85),
        adjust(vimwiki_header!("== comments =="), 86),
        blank_at_line(87),
        blank_at_line(88),
        blank_at_line(89),
        adjust(vimwiki_paragraph!(r#"this"#), 90),
        blank_at_line(91),
        adjust(vimwiki_paragraph!(r#"is "#), 92),
        blank_at_line(93),
        adjust(vimwiki_header!("== inline code =="), 94),
        blank_at_line(95),
        adjust(vimwiki_paragraph!(r#"Here is some `inline code`."#), 96),
        blank_at_line(97),
        adjust(vimwiki_paragraph!(r#"Just two backticks: ``"#), 98),
        blank_at_line(99),
        adjust(vimwiki_header!("== preformatted text =="), 100),
        blank_at_line(101),
        adjust(
            vimwiki_preformatted_text_raw! {r#"
{{{
  Tyger! Tyger! burning bright
   In the forests of the night,
    What immortal hand or eye
     Could frame thy fearful symmetry?
  In what distant deeps or skies
   Burnt the fire of thine eyes?
    On what wings dare he aspire?
     What the hand dare sieze the fire?
}}}
            "#},
            102,
        ),
        blank_at_line(112),
        adjust(
            vimwiki_header!("=== preformatted text with attributes ==="),
            113,
        ),
        blank_at_line(114),
        // TODO: Evaluate if we are switching from ; to space as separator
        //       since this won't parse as preformatted text currently
        // adjust(
        //     vimwiki_preformatted_text_raw! {r#"
        // {{{class="python" style="color:blue"
        // for i in range(1, 5):
        // print(i)
        // }}}
        //     "#},
        //     115,
        // ),
    ];

    // TODO: Validate comments
    compare_page_elements(&page.elements, &expected);
}
