use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_elements},
};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, Region, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::VimwikiWikiRelatedTools.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        vimwiki_header!("= Related Tools =")
            .take_with_region(Region::from((1, 1, 1, 18)))
            .into(),
        blank_line()
            .take_with_region(Region::from((2, 1, 2, 1))),
        vimwiki_paragraph! {r#"
            This page contains Vim plugins and external tools that can be used with Vimwiki.
            These lists are incomplete so please _help update them_ if you know of something
            that is missing!
        "#}
            .take_with_region(Region::from((3, 1, 5, 17)))
            .into(),
        blank_line()
            .take_with_region(Region::from((6, 1, 6, 1))),
        vimwiki_header!("== Vim Plugins ==")
            .take_with_region(Region::from((7, 1, 7, 18)))
            .into(),
        blank_line()
            .take_with_region(Region::from((8, 1, 8, 1))),
        vimwiki_list! {r#"
            - [[https://github.com/mattn/calendar-vim|calendar-vim]]
                - Select a date to open a diary page.
            - [[https://github.com/tbabej/taskwiki|taskwiki]]
                - Integration with [[https://taskwarrior.org/|taskwarrior]] for task
                  management. This only supports the default syntax.
            - [[https://github.com/majutsushi/tagbar|Tagbar]]
                - Creates a sidebar to view generated tags. For Vimwiki this can be
                  used to display links to headers.
                - For this to work an [[https://raw.githubusercontent.com/vimwiki/utils/master/vwtags.py|additional script]]
                  is needed which is available in the [[https://github.com/vimwiki/utils|utility repository]].
                  Additional instructions are in the comments of the script.
                - If using Markdown syntax `#` symbols used within code blocks are
                  recognized as a header i.e. in a shell script snippet. An alternative
                  version that works for Markdown can be found
                  [[https://raw.githubusercontent.com/jszakmeister/markdown2ctags/master/markdown2ctags.py|here]].
            - [[https://github.com/teranex/vimwiki-tasks|vimwiki-tasks]]
                - Another integration with [[https://taskwarrior.org/|taskwarrior]]. This
                  plugin does not seem to be maintained any longer.
            - [[https://github.com/matt-snider/vim-tagquery|vim-tagquery]]
                - A vim plugin that enables improved querying of tags.
                - This can be used to search for multiple tags.
                - See [[https://github.com/vimwiki/vimwiki/issues/723|Issue #723]]
        "#}
            .take_with_region(Region::from((9, 1, 30, 71)))
            .into(),
        blank_line()
            .take_with_region(Region::from((31, 1, 31, 1))),
        vimwiki_header!("== External Tools ==")
            .take_with_region(Region::from((32, 1, 32, 21)))
            .into(),
        blank_line()
            .take_with_region(Region::from((33, 1, 33, 1))),
        vimwiki_list! {r#"
            - [[https://github.com/vimwiki/vimwiki/blob/master/autoload/vimwiki/customwiki2html.sh|customwiki2html.sh]]
                - Script available in the official repository to convert Markdown to HTML.
                - See the comments for more information and usage instructions.
            - [[https://pandoc.org/MANUAL.html|Pandoc]]
                - Convert Vimwiki to various other formats such as HTML, PDF, etc.
            - [[https://github.com/patrickdavey/vimwiki_markdown|vimwiki_markdown]]
                - A Ruby gem to convert vimwiki Markdown files to HTML. Still being actively
                developed.
            - [[https://github.com/WnP/vimwiki_markdown|vimwiki_markdown Python]]
                - A Python script to convert Markdown files to HTML.
                - Also see [[https://github.com/vimwiki/vimwiki/issues/578|Issue #578]]
            - [[https://github.com/maqiv/vimwiki-godown#todo|vimwiki-godown]]
                - HTML converter that adds the ability to prefix relative links to other
                  Vimwiki pages
                - See [[https://github.com/vimwiki/vimwiki/issues/284|Issue #284]]
            - [[https://github.com/sstallion/vimwiki-skel|vimwiki-skel]]
                - Uses [[https://dynalon.github.io/mdwiki/#!index.md|MDWiki]] to convert
                Markdown files to HTML.
            - [[https://gist.github.com/maikeldotuk/54a91c21ed9623705fdce7bab2989742|wiki2html.sh Gist]]
                - Uses Pandoc to convert Markdown files to HTML.
            - [[https://github.com/fasheng/vimwiki2org|vimwiki2org]]
                - Convert vimwiki to [[https://orgmode.org/|Emacs orgmode]]. Has not been
                updated in 6 years.
            - [[https://github.com/ycpei/vwweb|vwweb]]
                - Python script to generate a website from Vimwiki files.
            - [[https://box.matto.nl/vimwikijabberbot.html|vimwiki-todo-jabberbot]]
                - Todo management using Jabberbot. The linked GitHub repository seems to
                have been removed.
            - [[https://github.com/zweifisch/vimwiki-tools|vimwiki-tools]]
                - Python tool to generate an index and convert wiki files to Markdown
                format. This has not been updated in 6 years.
        "#}
            .take_with_region(Region::from((34, 1, 64, 50)))
            .into(),
    ];

    compare_page_elements(&page.elements, &expected);
}
