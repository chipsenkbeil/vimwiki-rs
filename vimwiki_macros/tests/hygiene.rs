#![allow(dead_code)]
#![no_implicit_prelude]

use ::vimwiki_macros::*;

#[test]
fn hygiene_passes_for_vimwiki() {
    let _ = vimwiki_page!("some text");
    let _ = vimwiki_block_element!("some text");
    let _ = vimwiki_inline_element_container!("some text");
    let _ = vimwiki_inline_element!("some text");
    let _ = vimwiki_blockquote!("> some text");
    let _ = vimwiki_comment!("%% some comment");
    let _ = vimwiki_line_comment!("%% some comment");
    let _ = vimwiki_multi_line_comment!("%%+ some comment +%%");
    let _ = vimwiki_definition_list! {r#"
        term:: definition
        term2::
        :: def 2
        :: def 3
    "#};
    let _ = vimwiki_divider!("----");
    let _ = vimwiki_header!("= header =");
    let _ = vimwiki_link!("[[link]]");
    let _ = vimwiki_link!("[[diary:2012-03-05]]");
    let _ = vimwiki_link!("[[file:path/to/file]]");
    let _ = vimwiki_link!("[[wiki1:Some link]]");
    let _ = vimwiki_link!("[[wn.MyWiki:Some link]]");
    let _ = vimwiki_link!("https://example.com");
    let _ = vimwiki_link!("{{https://example.com/img.jpg}}");
    let _ = vimwiki_placeholder!("%date 2012-03-05");
    let _ = vimwiki_placeholder!("%nohtml");
    let _ = vimwiki_placeholder!("%other some text");
    let _ = vimwiki_placeholder!("%template my_template");
    let _ = vimwiki_placeholder!("%title some text");
    let _ = vimwiki_code_block! {r#"
        {{{
        some code
        }}}
    "#};
    let _ = vimwiki_table!("|cell|");
    let _ = vimwiki_tags!(":tag:");
    let _ = vimwiki_text!("some text");
    let _ = vimwiki_decorated_text!("*some text*");
    let _ = vimwiki_decorated_text!("_some text_");
    let _ = vimwiki_decorated_text!("~~some text~~");
    let _ = vimwiki_decorated_text!("^some text^");
    let _ = vimwiki_decorated_text!(",,some text,,");
    let _ = vimwiki_keyword!("TODO");
}
