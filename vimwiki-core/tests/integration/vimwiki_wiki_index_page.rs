use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::macros::*;
use vimwiki::*;

#[test]
fn test() {
    let contents = VimwikiFile::VimwikiWikiIndex.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();
    let expected = vec![
        vimwiki_header!("= Vimwiki Wiki =")
            .into(),
        vimwiki_paragraph!("*Welcome to the Vimwiki wiki!*")
            .into(),
        vimwiki_header!("== Official Repositories ==")
            .into(),
        vimwiki_paragraph!("Here are links to the official Vimwiki repositories:")
            .into(),
        vimwiki_list! {r#"
            - [[https://github.com/vimwiki/vimwiki|Vimwiki]]
            - [[https://vimwiki.github.io/vimwikiwiki/|Vimwiki Wiki]] (GitHub pages site built using Vimwiki)
                - [[https://github.com/vimwiki/vimwikiwiki|source repository]]
            - [[https://github.com/vimwiki/utils|Utilities]]
            - [[https://github.com/vimwiki/testwikis|Test wikis]] - sample wikis in each of
              the 3 syntax variants. Used for testing and development.
        "#}
            .into(),
        vimwiki_header!("== Wiki Pages ==")
            .into(),
        vimwiki_paragraph!("Checkout these pages for additional information and tips!")
            .into(),
        vimwiki_list! {r#"
            - [[Tips and Snips]]
            - [[Related Tools]]
            - [[Troubleshooting]]
        "#}
            .into(),
        vimwiki_header!("== Chat/Forums ==")
            .into(),
        vimwiki_list! {r#"
            - [[https://groups.google.com/forum/#!forum/vimwiki|Google Vimwiki group]]
            - [[https://webchat.freenode.net/?channels=#vimwiki|Freenode Webchat]]
        "#}
            .into(),
        vimwiki_header!("== Outdated Versions ==")
            .into(),
        vimwiki_paragraph! {r#"
            These links point to some old versions of Vimwiki that are no longer maintained.
            The original Vimwiki was hosted on Google Code which has since shutdown.
        "#}
            .into(),
        vimwiki_list! {r#"
                - [[https://code.google.com/archive/p/vimwiki/|Google Code Archive]]
                - [[https://github.com/vimwiki-backup/vimwiki|Google Code Backup on Github]]
                - [[https://www.vim.org/scripts/script.php?script_id=2226|Vimwiki on vim.org]]
            "#}
            .into(),
        vimwiki_header!("== Related Projects ==")
            .into(),
        vimwiki_list! {r#"
            - [[https://github.com/lervag/wiki.vim|wiki.vim]]
            - [[https://github.com/fcpg/vim-waikiki|vim-waikiki]]
            - [[https://github.com/jceb/vim-orgmode|vim-orgmode]]
            - [[https://github.com/tbabej/taskwiki|taskwiki]]
            - [[https://github.com/xolox/vim-notes|vim-notes]]
        "#}
            .into(),
    ];

    compare_page_elements(page.elements(), &expected);
}
