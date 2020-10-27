use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::{elements::*, Language};
use vimwiki_macros::*;

#[test]
fn test() {
    let contents = VimwikiFile::PandocVimwikiReader.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    let expected = vec![
        vimwiki_header!("= _*implemented*_ =").into(),
        vimwiki_header!("= header =").into(),
        vimwiki_header!("== header level two ==").into(),
        vimwiki_header!("=== header `level` 3 ===").into(),
        vimwiki_header!("==== header ~~level~~ four ====").into(),
        vimwiki_header!("===== header _*level* 5_ =====").into(),
        vimwiki_header!("====== header level 6 ======").into(),
        vimwiki_paragraph!("======= not a header ========").into(),
        vimwiki_paragraph!("hi== not a header ==").into(),
        vimwiki_paragraph!("=== not a header ==").into(),
        vimwiki_paragraph!("=== not a header ===-").into(),
        vimwiki_paragraph!("not a header:").into(),
        vimwiki_header!("=n=").into(),
        vimwiki_paragraph!("=== not a header ====").into(),
        vimwiki_header_raw!(" == centred header ==").into(),
        vimwiki_header!("== header with some `==` in between ==").into(),
        vimwiki_header!("== header with some == in between ==").into(),
        vimwiki_header!("== header with some ==in between ==").into(),
        vimwiki_header!("== emph strong and strikeout ==").into(),
        vimwiki_paragraph!("_emph_ *strong*").into(),
        vimwiki_paragraph!("*_strong and emph_*").into(),
        vimwiki_paragraph!("_*emph and strong*_").into(),
        vimwiki_paragraph!("*_emph inside_ strong*").into(),
        vimwiki_paragraph!("*strong with _emph_*").into(),
        vimwiki_paragraph!("_*strong inside* emph_").into(),
        vimwiki_paragraph!("_~~strikeout~~ inside emph_").into(),
        vimwiki_paragraph!("~~This is _struck out_ with emph~~ ").into(),
        vimwiki_paragraph! {r#"
            *not
            strong*
        "#}
        .into(),
        vimwiki_paragraph!("just two stars: **").into(),
        vimwiki_paragraph!("just two underscores: __").into(),
        vimwiki_paragraph!("just four ~s: ~~~~").into(),
        vimwiki_paragraph!("_not").into(),
        // TODO: Comment shows up here
        vimwiki_paragraph!("emph_").into(),
        vimwiki_paragraph!("~~not").into(),
        // TODO: Comment shows up here
        vimwiki_paragraph!("strikeout~~").into(),
        vimwiki_header!("== horizontal rule ==").into(),
        vimwiki_paragraph!("top").into(),
        vimwiki_divider!("----").into(),
        vimwiki_paragraph!("middle").into(),
        vimwiki_divider!("-------").into(),
        vimwiki_paragraph!("not a rule-----").into(),
        vimwiki_paragraph! {r#"
            not a rule (trailing spaces):
            -----
        "#}
        .into(),
        vimwiki_paragraph! {r#"
            not a rule (leading spaces):
              ----
        "#}
        .into(),
        vimwiki_header!("== comments ==").into(),
        vimwiki_paragraph!(r#"this"#).into(),
        vimwiki_paragraph!(r#"is "#).into(),
        vimwiki_header!("== inline code ==").into(),
        vimwiki_paragraph!(r#"Here is some `inline code`."#).into(),
        vimwiki_paragraph!(r#"Just two backticks: ``"#).into(),
        vimwiki_header!("== preformatted text ==").into(),
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
        "#}
        .into(),
        vimwiki_header!("=== preformatted text with attributes ===").into(),
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

    compare_page_elements(&page.elements, &expected);
}
