use super::fixtures::VimwikiFile;
use vimwiki::{elements::*, Language};

#[test]
fn test() {
    let language = Language::from_vimwiki_string(
        VimwikiFile::VimwikiSpecification.load().unwrap(),
    );
    let _page: Page = language.parse().unwrap();

    todo!();
}
