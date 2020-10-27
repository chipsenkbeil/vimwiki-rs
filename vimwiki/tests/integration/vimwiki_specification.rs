use super::fixtures::VimwikiFile;
use vimwiki::{elements::*, Language};

#[test]
fn test() {
    let contents = VimwikiFile::VimwikiSpecification.load().unwrap();
    let _page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();

    todo!();
}
