use super::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::{elements::*, Language, Region};
use vimwiki_macros::*;

#[test]
fn test() {
    let language = Language::from_vimwiki_string(
        VimwikiFile::VimwikiWikiTipsAndSnips.load().unwrap(),
    );
    let _page: Page = language.parse().unwrap();
    todo!();
    // let expected = vec![
    //     vimwiki_header!("= Tips and Snips =")
    //         .take_with_region(Region::from((1, 1, 1, 19)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Vimwiki cannot be all things to all users so here are some tips and code
    //         snippets you may find useful for customizing Vimwiki to your liking.
    //     "#}
    //         .take_with_region(Region::from((3, 1, 4, 69)))
    //         .into(),
    //     vimwiki_header!("== Cheat Sheet ==")
    //         .take_with_region(Region::from((6, 1, 6, 18)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         There are several cheat sheets for Vimwiki:
    //     "#}
    //         .take_with_region(Region::from((7, 1, 7, 44)))
    //         .into(),
    //     vimwiki_list! {r#"
    //         - [[http://thedarnedestthing.com/vimwiki%20cheatsheet|The Darnedest Thing - Vimwiki Cheatsheet]]
    //         - [[https://gist.github.com/drkarl/4c503bccb62558dc85e8b1bc0f29e9cb|Short Cheatsheet]]
    //         - [[https://dokk.org/library/Vimwiki_1.1.1_Quick_Reference_(Pospichal_2001)|PDF Cheatsheet incl. Syntax]]
    //     "#}
    //         .take_with_region(Region::from((8, 1, 10, 106)))
    //         .into(),
    //     vimwiki_header!("== Custom folding ==")
    //         .take_with_region(Region::from((12, 1, 12, 21)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Vimwiki has simple folding folding methods built in but lets you easily
    //         customize folds with the `g:vimwiki_folding` option. For example, if you prefer
    //         that the last blank line before a heading not get folded, add this to your
    //         `.vimrc` file:
    //     "#}
    //         .take_with_region(Region::from((14, 1, 17, 15)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         `let g:vimwiki_folding = 'custom'`
    //     "#}
    //         .take_with_region(Region::from((19, 1, 19, 35)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Then add the following to the `ftplugin/vimwiki.vim` plugin in your `.vim`
    //         configuration folder (create this file if it doesn't already exist):
    //     "#}
    //         .take_with_region(Region::from((21, 1, 22, 69)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // let l:vimwiki_fold_blank_lines = 0  " set to 1 to fold blank lines
    // let l:vimwiki_header_type = '#'     " set to '=' for wiki syntax
    // setlocal foldlevel=1
    // setlocal foldenable
    // setlocal foldmethod=expr
    // setlocal foldexpr=Fold(v:lnum)

    // function! Fold(lnum)
    // let fold_level = strlen(matchstr(getline(a:lnum), '^' . l:vimwiki_header_type . '\+'))
    // if (fold_level)
    //  return '>' . fold_level  " start a fold level
    // endif
    // if getline(a:lnum) =~? '\v^\s*$'
    //  if (strlen(matchstr(getline(a:lnum + 1), '^' . l:vimwiki_header_type . '\+')) > 0 && !g:vimwiki_fold_blank_lines)
    //    return '-1' " don't fold last blank line before header
    //  endif
    // endif
    // return '=' " return previous fold level
    // endfunction
    // }}}
    //     "#}
    //         .take_with_region(Region::from((24, 1, 44, 4)))
    //         .into(),
    //     vimwiki_header!("== Task Management ==")
    //         .take_with_region(Region::from((46, 1, 46, 22)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Vimwiki makes it effortless to add tasks to any wiki page. Unfortunately,
    //         this means that your tasks get dispersed rather widely, especially if
    //         you're tracking action items from meeting notes in your diary. The snippets
    //         below make it easier to manage tasks in vimwiki without adding any additional
    //         plugins or relying on external task management tools.
    //     "#}
    //         .take_with_region(Region::from((48, 1, 52, 54)))
    //         .into(),
    //     vimwiki_header!("=== Find Incomplete Tasks ===")
    //         .take_with_region(Region::from((54, 1, 54, 31)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         The following will open a QuickFix window with incomplete tasks, but only those
    //         which are in a hyphenated (`-`) list. This is a simple way to filter only on
    //         tasks which are ready to be performed.
    //     "#}
    //         .take_with_region(Region::from((56, 1, 58, 39)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // function! VimwikiFindIncompleteTasks()
    // lvimgrep /- \[ \]/ %:p
    // lopen
    // endfunction

    // function! VimwikiFindAllIncompleteTasks()
    // VimwikiSearch /- \[ \]/
    // lopen
    // endfunction

    // nmap <Leader>wa :call VimwikiFindAllIncompleteTasks()<CR>
    // nmap <Leader>wx :call VimwikiFindIncompleteTasks()<CR>
    // }}}
    //     "#}
    //         .take_with_region(Region::from((60, 1, 73, 4)))
    //         .into(),
    //     vimwiki_header!("== Encrypting Vimwiki pages ==")
    //         .take_with_region(Region::from((75, 1, 75, 31)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         If you want to encrypt singe pages of your wiki you can use [[https://github.com/jamessan/vim-gnupg|vim gnupg]] in
    //         conjunction with vimwiki. Add the following to your `vimrc`:
    //     "#}
    //         .take_with_region(Region::from((77, 1, 78, 61)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // let g:GPGFilePattern = '*.\(gpg\|asc\|pgp\)\(.wiki\)\='
    // }}}
    //     "#}
    //         .take_with_region(Region::from((80, 1, 82, 4)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Then you can create a link to a page in the following form: `[[link.asc]]`, the
    //         resulting file "link.asc.wiki" will be transparently encrypted by vim-gnupg.
    //         vim-gnupg will ask you to choose a key and gpg-agent will ask you to unlock the
    //         chosen key.
    //     "#}
    //         .take_with_region(Region::from((84, 1, 87, 12)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Note: If you use a different file-extension for your wikipages make sure to
    //         change the code above accordingly.
    //     "#}
    //         .take_with_region(Region::from((89, 1, 90, 35)))
    //         .into(),
    //     vimwiki_header!("== Cite entries from bibtex library ==")
    //         .take_with_region(Region::from((92, 1, 92, 39)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Vimwiki has no support built in yet, but see [[https://github.com/vimwiki/vimwiki/issues/361|this issue]] for workarounds.
    //     "#}
    //         .take_with_region(Region::from((93, 1, 93, 123)))
    //         .into(),
    //     vimwiki_header!("== FAQ ==")
    //         .take_with_region(Region::from((95, 1, 95, 10)))
    //         .into(),
    //     vimwiki_header!("=== General ===")
    //         .take_with_region(Region::from((97, 1, 97, 16)))
    //         .into(),
    //     vimwiki_header!("==== How to change the folder of the wiki? ====")
    //         .take_with_region(Region::from((98, 1, 98, 48)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         You have to configure your wiki(s) in your vimrc, then you can configure among
    //         other the folder.
    //     "#}
    //         .take_with_region(Region::from((99, 1, 100, 18)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // let g:vimwiki_list = [{'path': '~/mywiki/',
    //                   \ 'path_html': '~/mywiki_html'}]
    // }}}
    //     "#}
    //         .take_with_region(Region::from((102, 1, 105, 4)))
    //         .into(),
    //     vimwiki_header!("==== Can I start Vimwiki directly from shell? ====")
    //         .take_with_region(Region::from((107, 1, 107, 51)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Yes:
    //     "#}
    //         .take_with_region(Region::from((108, 1, 108, 5)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{bash
    // $ vim -c VimwikiIndex
    // }}}
    //     "#}
    //         .take_with_region(Region::from((110, 1, 112, 4)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Opening the file of a wikipage also does the trick, that way you can open it
    //         with another than your main page. Example:
    //     "#}
    //         .take_with_region(Region::from((114, 1, 115, 43)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{bash
    // $ alias importantpage='vim vimwiki/importantpage.wiki'
    // $ importantpage
    // }}}
    //     "#}
    //         .take_with_region(Region::from((117, 1, 120, 4)))
    //         .into(),
    //     vimwiki_header!("==== Useful shell function for git integration and launch ====")
    //         .take_with_region(Region::from((122, 1, 122, 63)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         If you init your vimwiki directory as a git repo, and add the following function
    //         to your `.bashrc` or `.zshrc`, you can interact with the repo using the command
    //         `vimwiki git [commands]` from any directory:
    //     "#}
    //         .take_with_region(Region::from((124, 1, 126, 45)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{bash
    // vimwiki () {
    // if [[ $# == 0 ]]
    // then
    //     nvim +'VimwikiIndex'
    // elif [[ $1 == 'git' ]]
    // then
    //     git -C ~/vimwiki/ ${@:2}
    // else
    //     echo 'Usage: vimwiki [git] [args ...]'
    // fi
    // }
    // }}}
    //     "#}
    //         .take_with_region(Region::from((128, 1, 140, 4)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         In addition, calling `vimwiki` without a git subcommand will automatically open
    //         the index.
    //     "#}
    //         .take_with_region(Region::from((142, 1, 143, 11)))
    //         .into(),
    //     vimwiki_header!("=== Markdown ===")
    //         .take_with_region(Region::from((145, 1, 145, 17)))
    //         .into(),
    //     vimwiki_header!("==== How do I use markdown syntax for my wikis? ====")
    //         .take_with_region(Region::from((147, 1, 147, 53)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         You have to configure your wiki(s) in your vimrc, then you can configure syntax
    //         and file extension. To set them to markdown and `.md` add the following
    //         configuration to you vimrc:
    //     "#}
    //         .take_with_region(Region::from((148, 1, 150, 28)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // let g:vimwiki_list = [{'path': '~/vimwiki/',
    //                   \ 'syntax': 'markdown', 'ext': '.md'}]
    // }}}
    //     "#}
    //         .take_with_region(Region::from((152, 1, 155, 4)))
    //         .into(),
    //     vimwiki_header!("==== Vimwiki considers every markdown-file as a wiki file ====")
    //         .take_with_region(Region::from((157, 1, 157, 63)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Vimwiki has a feature called "Temporary Wikis", that will treat every file with
    //         configured file-extension as a wiki. To disable this feature add this to your vimrc:
    //     "#}
    //         .take_with_region(Region::from((159, 1, 160, 85)))
    //         .into(),
    //     vimwiki_preformatted_text_raw! {r#"
    // {{{vim
    // let g:vimwiki_global_ext = 0
    // }}}
    //     "#}
    //         .take_with_region(Region::from((162, 1, 164, 4)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         Alternative you can set vimwiki to use markdown syntax but a different
    //         file-extension, like the default `.wiki`.
    //     "#}
    //         .take_with_region(Region::from((166, 1, 167, 42)))
    //         .into(),
    //     vimwiki_header!("== Got Other Great Ideas You'd Like to Share? ==")
    //         .take_with_region(Region::from((169, 1, 169, 49)))
    //         .into(),
    //     vimwiki_paragraph! {r#"
    //         If you have other snippets you find useful, please share them here on the wiki.
    //     "#}
    //         .take_with_region(Region::from((171, 1, 171, 80)))
    //         .into(),
    // ];

    // compare_page_elements(&page.elements, &expected);
}
