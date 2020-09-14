use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_components},
};
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, Region, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::VimwikiWikiIndex.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        vimwiki_header!("= Vimwiki Wiki =")
            .take_with_region(Region::from((0, 0, 0, 16)))
            .into(),
        blank_line()
            .take_with_region(Region::from((1, 0, 1, 0))),
        vimwiki_paragraph!("*Welcome to the Vimwiki wiki!*")
            .take_with_region(Region::from((2, 0, 2, 30)))
            .into(),
        blank_line()
            .take_with_region(Region::from((3, 0, 3, 0))),
        vimwiki_header!("== Official Repositories ==")
            .take_with_region(Region::from((4, 0, 4, 27)))
            .into(),
        blank_line()
            .take_with_region(Region::from((5, 0, 5, 0))),
        vimwiki_paragraph!("Here are links to the official Vimwiki repositories:")
            .take_with_region(Region::from((6, 0, 6, 52)))
            .into(),
        blank_line()
            .take_with_region(Region::from((7, 0, 7, 0))),
        vimwiki_list! {r#"
            - [[https://github.com/vimwiki/vimwiki|Vimwiki]]
            - [[https://vimwiki.github.io/vimwikiwiki/|Vimwiki Wiki]] (GitHub pages site built using Vimwiki)
                - [[https://github.com/vimwiki/vimwikiwiki|source repository]]
            - [[https://github.com/vimwiki/utils|Utilities]]
            - [[https://github.com/vimwiki/testwikis|Test wikis]] - sample wikis in each of
              the 3 syntax variants. Used for testing and development.
        "#}
            .take_with_region(Region::from((8, 0, 13, 58)))
            .into(),
        blank_line()
            .take_with_region(Region::from((14, 0, 14, 0))),
        vimwiki_header!("== Wiki Pages ==")
            .take_with_region(Region::from((15, 0, 15, 16)))
            .into(),
        blank_line()
            .take_with_region(Region::from((16, 0, 16, 0))),
        vimwiki_paragraph!("Checkout these pages for additional information and tips!")
            .take_with_region(Region::from((17, 0, 17, 57)))
            .into(),
        blank_line()
            .take_with_region(Region::from((18, 0, 18, 0))),
        vimwiki_list! {r#"
            - [[Tips and Snips]]
            - [[Related Tools]]
            - [[Troubleshooting]]
        "#}
            .take_with_region(Region::from((19, 0, 21, 21)))
            .into(),
        blank_line()
            .take_with_region(Region::from((22, 0, 22, 0))),
        vimwiki_header!("== Chat/Forums ==")
            .take_with_region(Region::from((23, 0, 23, 17)))
            .into(),
        blank_line()
            .take_with_region(Region::from((24, 0, 24, 0))),
        vimwiki_list! {r#"
            - [[https://groups.google.com/forum/#!forum/vimwiki|Google Vimwiki group]]
            - [[https://webchat.freenode.net/?channels=#vimwiki|Freenode Webchat]]
        "#}
            .take_with_region(Region::from((25, 0, 26, 70)))
            .into(),
        blank_line()
            .take_with_region(Region::from((27, 0, 27, 0))),
        vimwiki_header!("== Outdated Versions ==")
            .take_with_region(Region::from((28, 0, 28, 23)))
            .into(),
        blank_line()
            .take_with_region(Region::from((29, 0, 29, 0))),
        vimwiki_paragraph! {r#"
            These links point to some old versions of Vimwiki that are no longer maintained.
            The original Vimwiki was hosted on Google Code which has since shutdown.
        "#}
            .take_with_region(Region::from((30, 0, 31, 72)))
            .into(),
        blank_line()
            .take_with_region(Region::from((32, 0, 32, 0))),
        vimwiki_list! {r#"
                - [[https://code.google.com/archive/p/vimwiki/|Google Code Archive]]
                - [[https://github.com/vimwiki-backup/vimwiki|Google Code Backup on Github]]
                - [[https://www.vim.org/scripts/script.php?script_id=2226|Vimwiki on vim.org]]
            "#}
            .take_with_region(Region::from((33, 0, 35, 78)))
            .into(),
        blank_line()
            .take_with_region(Region::from((36, 0, 36, 0))),
        vimwiki_header!("== Related Projects ==")
            .take_with_region(Region::from((37, 0, 37, 22)))
            .into(),
        blank_line()
            .take_with_region(Region::from((38, 0, 38, 0))),
        vimwiki_list! {r#"
            - [[https://github.com/lervag/wiki.vim|wiki.vim]]
            - [[https://github.com/fcpg/vim-waikiki|vim-waikiki]]
            - [[https://github.com/jceb/vim-orgmode|vim-orgmode]]
            - [[https://github.com/tbabej/taskwiki|taskwiki]]
            - [[https://github.com/xolox/vim-notes|vim-notes]]
        "#}
            .take_with_region(Region::from((39, 0, 43, 50)))
            .into(),
    ];

    compare_page_components(&page.components, &expected);
}
