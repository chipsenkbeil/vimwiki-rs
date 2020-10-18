use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::{elements::*, Language, Region};
use vimwiki_macros::*;

#[test]
fn test() {
    let language = Language::from_vimwiki_string(
        VimwikiFile::VimwikiWikiTroubleshooting.load().unwrap(),
    );
    let page: Page = language.parse().unwrap();
    let expected = vec![
        vimwiki_header!("= Troubleshooting =")
            .take_with_region(Region::from((1, 1, 1, 20)))
            .into(),
        vimwiki_header!("== Minimal Vimrc ==")
            .take_with_region(Region::from((3, 1, 3, 20)))
            .into(),
        vimwiki_paragraph! {r#"
            These steps might help to determine if an issue is related to your
            setup/configuration or if the problem is a bug in Vimwiki.
        "#}
            .take_with_region(Region::from((5, 1, 6, 59)))
            .into(),
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

    compare_page_elements(&page.elements, &expected);
}
