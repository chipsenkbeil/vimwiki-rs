use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, LE};
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

#[test]
fn test() {
    vimwiki::timekeeper::enable();
    let page: LE<Page> =
        RawStr::Vimwiki(&VimwikiFile::PandocVimwikiReader.load().unwrap())
            .try_into()
            .unwrap();
    vimwiki::timekeeper::print_report(true);
    vimwiki::timekeeper::disable();

    let expected = vec![
        adjust(vimwiki_header!("= _*implemented*_ ="), 1),
        adjust(vimwiki_header!("= header ="), 2),
        adjust(vimwiki_header!("== header level two =="), 4),
        adjust(vimwiki_header!("=== header `level` 3 ==="), 6),
        adjust(vimwiki_header!("==== header ~~level~~ four ===="), 8),
        adjust(vimwiki_header!("===== header _*level* 5_ ====="), 10),
        adjust(vimwiki_header!("====== header level 6 ======"), 12),
        adjust(vimwiki_paragraph!("======= not a header ========"), 14),
        adjust(vimwiki_paragraph!("hi== not a header =="), 16),
        adjust(vimwiki_paragraph!("=== not a header =="), 18),
        adjust(vimwiki_paragraph!("=== not a header ===-"), 20),
        adjust(vimwiki_paragraph!("not a header:"), 22),
        adjust(vimwiki_header!("=n="), 24),
        adjust(vimwiki_paragraph!("=== not a header ===="), 26),
        adjust(vimwiki_header_raw!(" == centred header =="), 28),
        adjust(
            vimwiki_header!("== header with some `==` in between =="),
            30,
        ),
        adjust(vimwiki_header!("== header with some == in between =="), 31),
        adjust(vimwiki_header!("== header with some ==in between =="), 32),
        adjust(vimwiki_header!("== emph strong and strikeout =="), 34),
        adjust(vimwiki_paragraph!("_emph_ *strong*"), 36),
        adjust(vimwiki_paragraph!("*_strong and emph_*"), 38),
        adjust(vimwiki_paragraph!("_*emph and strong*_"), 40),
        adjust(vimwiki_paragraph!("*_emph inside_ strong*"), 42),
        adjust(vimwiki_paragraph!("*strong with _emph_*"), 44),
        adjust(vimwiki_paragraph!("_*strong inside* emph_"), 46),
        adjust(vimwiki_paragraph!("_~~strikeout~~ inside emph_"), 48),
        adjust(
            vimwiki_paragraph!("~~This is _struck out_ with emph~~ "),
            50,
        ),
        adjust(
            vimwiki_paragraph! {r#"
            *not
            strong*
        "#},
            52,
        ),
        adjust(vimwiki_paragraph!("just two stars: **"), 55),
        adjust(vimwiki_paragraph!("just two underscores: __"), 57),
        adjust(vimwiki_paragraph!("just four ~s: ~~~~"), 59),
        adjust(vimwiki_paragraph!("_not"), 61),
        adjust(vimwiki_paragraph!("emph_"), 63),
        adjust(vimwiki_paragraph!("~~not"), 65),
        adjust(vimwiki_paragraph!("strikeout~~"), 68),
        adjust(vimwiki_header!("== horizontal rule =="), 70),
        adjust(vimwiki_paragraph!("top"), 72),
        adjust(vimwiki_divider!("----"), 73),
        adjust(vimwiki_paragraph!("middle"), 74),
        adjust(vimwiki_divider!("-------"), 76),
        adjust(vimwiki_paragraph!("not a rule-----"), 78),
        adjust(
            vimwiki_paragraph! {r#"
                not a rule (trailing spaces):
                ----- 
            "#},
            80,
        ),
        adjust(
            vimwiki_paragraph! {r#"
                not a rule (leading spaces):
                  ----
            "#},
            83,
        ),
        adjust(vimwiki_header!("== comments =="), 86),
        adjust(vimwiki_paragraph!(r#"this"#), 90),
        adjust(vimwiki_paragraph!(r#"is "#), 92),
        adjust(vimwiki_header!("== inline code =="), 94),
        adjust(vimwiki_paragraph!(r#"Here is some `inline code`."#), 96),
        adjust(vimwiki_paragraph!(r#"Just two backticks: ``"#), 98),
        adjust(vimwiki_header!("== preformatted text =="), 100),
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
        adjust(
            vimwiki_header!("=== preformatted text with attributes ==="),
            113,
        ),
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
