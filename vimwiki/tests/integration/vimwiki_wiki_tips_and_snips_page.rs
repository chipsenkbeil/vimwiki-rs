use super::fixtures::VimwikiFile;
use vimwiki::{Parser, VimwikiParser};

#[test]
fn test() {
    let _page = VimwikiParser::parse_str(
        &VimwikiFile::VimwikiWikiTipsAndSnips.load().unwrap(),
    )
    .unwrap();
    todo!();
}
