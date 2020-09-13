use vimwiki::components::*;
use vimwiki_macros::vimwiki;

#[test]
fn vimwiki_should_convert_syntax_to_page() {
    let vimwiki_page = vimwiki! {r#"
    = Some header =

    A paragraph of text
    starts here.

    - List item 1
        - Sub item 1
        - Sib item 2
    - List item 2
    "#};
    println!("{:?}", vimwiki_page);
    todo!();
}
