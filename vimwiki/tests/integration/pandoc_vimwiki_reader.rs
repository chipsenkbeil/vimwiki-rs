use super::fixtures::VimwikiFile;
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, LC};

#[test]
fn test() {
    let _page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::PandocVimwikiReader.load().unwrap())
            .try_into()
            .unwrap();
    todo!();
}
