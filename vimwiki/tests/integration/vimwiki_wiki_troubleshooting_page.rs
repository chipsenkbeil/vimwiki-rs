use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_components},
};
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, Region, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LC<Page> = RawStr::Vimwiki(
        &VimwikiFile::VimwikiWikiTroubleshooting.load().unwrap(),
    )
    .try_into()
    .unwrap();
    let expected = vec![
        vimwiki_header!("= Troubleshooting =")
            .take_with_region(Region::from((1, 1, 1, 20)))
            .into(),
        blank_line()
            .take_with_region(Region::from((2, 1, 2, 1))),
        vimwiki_header!("== Minimal Vimrc ==")
            .take_with_region(Region::from((3, 1, 3, 20)))
            .into(),
        blank_line()
            .take_with_region(Region::from((4, 1, 4, 1))),
        vimwiki_paragraph! {r#"
            These steps might help to determine if an issue is related to your
            setup/configuration or if the problem is a bug in Vimwiki.
        "#}
            .take_with_region(Region::from((5, 1, 6, 59)))
            .into(),
        blank_line()
            .take_with_region(Region::from((7, 1, 7, 1))),
        vimwiki_list! {r#"
            1. Clone a fresh copy of the `dev` branch.
                {{{sh
                cd $HOME
                mkdir vw_tmp
                cd vw_tmp
                git clone -b dev --single-branch https://github.com/vimwiki/vimwiki.git
                }}}
            2. Create a minimal `vimrc` (you should still be in `$HOME/vw_tmp`)
                - `vim min_vimrc`
                - Paste the below text into the opened file. Add any additional Vimwiki
                  settings that are relevant to the issue being tested but keep in minimal!
                {{{vim
                set nocompatible
                filetype plugin on
                syntax on
                set runtimepath+=~/vw_tmp/vimwiki
                let wiki = {}
                let wiki.path = '~/vw_tmp/wiki'
                let wiki.path_html = '~/vw_tmp/wiki/html'
                let wiki.syntax = 'default'
                let wiki.ext = '.wiki'
                let g:vimwiki_list = [wiki]
                }}}
            3. Start vim using the `min_vimrc`
                - `vim -u min_vimrc`
            4. Open up Vimwiki (`\ww`) and verify the issue still occurs.
        "#}
            .take_with_region(Region::from((8, 1, 33, 62)))
            .into(),
    ];

    compare_page_components(&page.components, &expected);
}
