use super::fixtures::VimwikiFile;
use vimwiki::{elements::*, Language};

#[test]
fn test() {
    vimwiki::timekeeper::enable();
    let language = Language::from_vimwiki_string(
        VimwikiFile::VimwikiSpecification.load().unwrap(),
    );
    let _page: Page = language.parse().unwrap();
    vimwiki::timekeeper::print_report(true);
    vimwiki::timekeeper::disable();

    todo!();
}
