use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::{elements::*, Language};
use vimwiki_macros::*;

#[test]
#[ignore]
fn test() {
    let contents = VimwikiFile::PandocVimwikiReader.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    macro_rules! make_link {
        ($link:expr) => {
            Located::from(Paragraph::from(
                vimwiki_link!($link).map(InlineElement::from),
            ))
            .into()
        };
    }

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
        vimwiki_paragraph! {r#"
            _not
            %%comment
            emph_
        "#}
        .into(),
        // TODO: Should the strikeout be one paragraph with comments inbetween?
        //       Currently, if there is enough indentation, a blockquote is
        //       formed immediately after a paragraph
        vimwiki_paragraph!(r"~~not").into(),
        vimwiki_blockquote_raw!("    %%comment").into(),
        vimwiki_paragraph! {r#"
              %%comment
            strikeout~~
        "#}
        .into(),
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
        vimwiki_paragraph!("%% you can't see me.").into(),
        vimwiki_paragraph! {r#"
            this 
            %% secret
            is %% not secret
        "#}
        .into(),
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
        vimwiki_preformatted_text_raw! {r#"
 {{{class="python" style="color:blue"
 for i in range(1, 5):
     print(i)
 }}}
        "#}
        .into(),
        vimwiki_header!("=== preformatted text with nested syntax ===").into(),
        vimwiki_preformatted_text! {r#"
            {{{sql
            SELECT * FROM table
            }}}
        "#}
        .into(),
        vimwiki_header!("=== empty preformatted text ===").into(),
        vimwiki_preformatted_text! {r#"
            {{{
            }}}
        "#}
        .into(),
        vimwiki_header!("== block quotes ==").into(),
        vimwiki_blockquote_raw! {r#"
    (indentation 4 spaces) This would be a blockquote in Vimwiki. It is not *highlighted* in Vim but
    (indentation 1 space followed by 1 tab of width 4) could be styled by CSS in HTML. Blockquotes are usually used to quote a
    (indentation 1 tab of width 4) long piece of text from another source. ~~blah blah~~ :blockquote:
        "#}.into(),
        vimwiki_header!("== external links ==").into(),
        make_link!("[[http://google.com|_Google_ search engine]]"),
        make_link!("http://pandoc.org"),
        make_link!("ftp://vim.org"),
        Located::from(Paragraph::from(vec![
            vimwiki_link!("[[http://google.com]]").map(InlineElement::from),
            Located::from(Text::from(" ")).map(InlineElement::from),
        ])).into(),
        make_link!("[[mailto:info@example.org|email me]]"),
        make_link!("mailto:hello@bye.com"),
        vimwiki_header!("== internal links ==").into(),
        make_link!("[[This is a link]]"),
        make_link!("[[This is a link source|Description of the link]]"),
        make_link!("[[projects/Important Project 1]]"),
        make_link!("[[../index]]"),
        make_link!("[[a subdirectory/|Other files]]"),
        make_link!("[[#tag-one|try me to test tag anchors]]"),
        make_link!("[[#block quotes|try me to test header anchors]]"),
        make_link!("[[#strong|try me to test strong anchors]]"),
        make_link!("[[Todo List#Tomorrow|Tasks for tomorrow]]"),
        make_link!("[[diary:2017-05-01]]"),
        make_link!("[[file:../assets/data.csv|Important Data]]"),
        vimwiki_header!("=== links with thumbnails ===").into(),
        make_link!("[[http://www.google.com|{{./movie.jpg}}]]"),
        vimwiki_header!("== images ==").into(),
        make_link!("{{file:./lalune.jpg}}"),
        make_link!("{{http://vimwiki.googlecode.com/hg/images/vimwiki_logo.png|Vimwiki}}"),
        Located::from(Paragraph::from(vec![
            vimwiki_link!("{{local:./movie.jpg}}").map(InlineElement::from),
            Located::from(Text::from("  ")).map(InlineElement::from),
        ])).into(),
        vimwiki_header!("=== image with attributes ===").into(),
        // TODO: Should these parse as transclusion links? Currently, we
        //       enforce a scheme here, which is why they are not links
        vimwiki_paragraph!(r#"{{lalune.jpg|_cool stuff_|style="width:150px;height:120px;"}}"#).into(),
        vimwiki_paragraph!(r#"{{nonexist.jpg|*Non-existing* image|class="center flow blabla" style="font-color:red"}}"#).into(),
        vimwiki_paragraph!(r#"{{lalune.jpg|_cool stuff_|style="width:150px;height:120px;"|anything in this segment is ignored}}"#).into(),
        vimwiki_header!("== lists ==").into(),
    ];

    //     r#"
    // # ordered list item 1, and here is some math belonging to list item 1
    //  {{$
    //  a^2 + b^2 = c^2
    //  }}$
    //   and some preformatted and tables belonging to item 1 as well
    // {{{
    // I'm part of item 1.
    // }}}
    // | this table  | is        |
    // | also a part | of item 1 |
    //  and some more text belonging to item 1.
    // # ordered list item 2

    // * Bulleted list item 1
    // * Bulleted list item 2

    // # Bulleted list item 1
    // # the # become numbers when converted to HTML

    // - Bulleted list item 1
    // - Bulleted list item 2

    // * Item 1
    // * Item 2
    //     # Sub item 1 (indentation 4 spaces)
    //   Sub item 1 continued line.
    // %%comments
    //     Sub item 1 next continued line.
    //     * Sub item 2, as an ordered list item even though the identifier is `*` (indentation 2 spaces followed by one tab of width 4)
    //     * etc.
    //  Continuation of Item 2
    //  Next continuation of Item 2
    // But this is a new paragraph.

    // # 1
    //     * `1.1`
    //   * 2
    //       * 2.1
    //  * 3

    // === ordered lists with non-# identifiers ===
    // 1. Numbered list item 1
    // 2. Numbered list item 2
    // 3. Numbered list item 3

    // 4. Numbered list item 1
    // 5. Numbered list item 2
    // 6. Numbered list item 3

    // 1) Numbered list item 1
    // 2) Numbered list item 2
    // 3) Numbered list item 3

    // a) Numbered list item 1
    // b) Numbered list item 2
    // c) Numbered list item 3

    // A) Numbered list item 1
    // B) Numbered list item 2
    // C) Numbered list item 3

    // i) Numbered list item 1
    // ii) Numbered list item 2
    // iii) Numbered list item 3

    // I) Numbered list item 1
    // II) Numbered list item 2
    // III) Numbered list item 3

    // - Bulleted list item 1
    // - Bulleted list item 2
    //   a) Numbered list sub item 1
    //   b) more ...
    //     * and more ...
    //     * ...
    //   c) Numbered list sub item 3
    //     1. Numbered list sub sub item 1
    //     2. Numbered list sub sub item 2
    //   d) etc.
    // - Bulleted list item 3

    // == todo lists ==
    // * [ ] task 1
    //     1. [.] 5
    // * [o] 3
    // * [] not a todo item
    // * [ ]not a todo item
    // * [r] not a todo item
    // * [     ] not a todo item
    // * [o] a tab in the todo list marker `[ ]`
    //     III) [O] 4
    //       5
    //     i) [X]
    // | a | b |
    // * [X] task 2

    // == math ==

    // $ \sum_i a_i^2 = 1 $

    // {{$
    // \sum_i a_i^2
    // =
    // 1
    // }}$

    // {{$%align%
    // \sum_i a_i^2 &= 1 + 1 \\
    // &= 2.
    // }}$

    // edge case (the `c^2 + ` after the multline tag is in the equation):
    // {{$%multline%c^2 +
    // a^2 + b^2
    // }}$

    // edge case (the tag is `hello%bye`)
    // {{$%hello%bye%
    // \int_a^b f(x) dx
    // }}$

    // Just two dollar signs: $$

    // [not math] You have $1
    // and I have $1.

    // == tags ==

    // :tag-one:tag-two:

    // == tables ==

    // | Year | Temperature (low) | Temperature (high) |
    // |------|-------------------|--------------------|
    // | 1900 | -10               | 25                 |
    // | 1910 | -15               | 30                 |
    // | 1920 | -10               | 32                 |
    // | 1930 | _N/A_             | _N/A_              |
    // | 1940 | -2                | 40                 |

    // === centered headerless tables ===
    //  | a | b |
    //  | c | d |

    // == paragraphs ==

    // This is first paragraph
    // with two lines.

    // This is a second paragraph with
    // two lines after many blank lines.

    // == definition list ==

    // Term 1:: Definition 1
    // Term 2::
    // :: Definition 2
    //   :: Definition 3
    // Term :: *separated* by :: _double colons_ :: Def1
    // :: Def2
    // Term with lots of trailing colons::::::::: Definition
    // :: This is :: A term (rather than a definition) :: and this is a definition
    // Term Without definitions ::
    // ::
    // Part :: of :: dt :: part of ::dd

    // :: Definition 1 without a term
    // :: Definition 2 without a term

    // T1 :: D1
    // new paragraph
    // T1 :: D1

    // Not::Definition

    // Not ::Definition

    // ::Not definition

    //     :: blockquote

    //     block :: quote

    // == metadata placeholders ==
    // %title title
    // %date 2017-05-01

    // %title second title is ignored
    // %date second date is ignored

    // %this is not a placeholder

    // placeholders
    // %title another title
    // %date 2017-04-23
    // serves as space / softbreak in paragraphs

    // == sup, sub ==

    // super^script^

    // sub,,script,,

    // == the todo mark ==
    // TODO:

    // = _*not implemented yet*_ =
    // == tables with spans ==
    // | a  | b  | c | d |
    // | \/ | e  | > | f |
    // | \/ | \/ | > | g |
    // | h  | >  | > | > |

    // == tables with multiple lines of headers ==
    // | a | b |
    // | c | d |
    // |---|---|

    // == some other placeholders ==
    // `template` placeholder is ignored.
    // %template template

    // `nohtml` placeholder is ignored.
    // %nohtml
    // "#;

    compare_page_elements(page.elements(), &expected);
}
