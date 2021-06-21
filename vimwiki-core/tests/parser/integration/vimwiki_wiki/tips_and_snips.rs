use crate::parser::{fixtures::VimwikiFile, utils::compare_page_elements};
use vimwiki::macros::*;
use vimwiki::*;

#[test]
fn test() {
    let contents = VimwikiFile::VimwikiWikiTipsAndSnips.load().unwrap();
    let page: Page = Language::from_vimwiki_str(&contents).parse().unwrap();
    let expected = vec![
        vimwiki_header!("= Tips and Snips =")
            .into(),
        vimwiki_paragraph! {r#"
            Vimwiki cannot be all things to all users so here are some tips and code
            snippets you may find useful for customizing Vimwiki to your liking.
        "#}
            .into(),
        vimwiki_header!("== Cheat Sheet ==")
            .into(),
        vimwiki_paragraph! {r#"
            There are several cheat sheets for Vimwiki:
        "#}
            .into(),
        vimwiki_list! {r#"
            - [[http://thedarnedestthing.com/vimwiki%20cheatsheet|The Darnedest Thing - Vimwiki Cheatsheet]]
            - [[https://gist.github.com/drkarl/4c503bccb62558dc85e8b1bc0f29e9cb|Short Cheatsheet]]
            - [[https://dokk.org/library/Vimwiki_1.1.1_Quick_Reference_(Pospichal_2001)|PDF Cheatsheet incl. Syntax]]
        "#}
            .into(),
        vimwiki_header!("== Custom folding ==")
            .into(),
        vimwiki_paragraph! {r#"
            Vimwiki has simple folding folding methods built in but lets you easily
            customize folds with the `g:vimwiki_folding` option. For example, if you prefer
            that the last blank line before a heading not get folded, add this to your
            `.vimrc` file:
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            `let g:vimwiki_folding = 'custom'`
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            Then add the following to the `ftplugin/vimwiki.vim` plugin in your `.vim`
            configuration folder (create this file if it doesn't already exist):
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
let l:vimwiki_fold_blank_lines = 0  " set to 1 to fold blank lines
let l:vimwiki_header_type = '#'     " set to '=' for wiki syntax
setlocal foldlevel=1
setlocal foldenable
setlocal foldmethod=expr
setlocal foldexpr=Fold(v:lnum)

 function! Fold(lnum)
   let fold_level = strlen(matchstr(getline(a:lnum), '^' . l:vimwiki_header_type . '\+'))
   if (fold_level)
     return '>' . fold_level  " start a fold level
   endif
   if getline(a:lnum) =~? '\v^\s*$'
     if (strlen(matchstr(getline(a:lnum + 1), '^' . l:vimwiki_header_type . '\+')) > 0 && !g:vimwiki_fold_blank_lines)
       return '-1' " don't fold last blank line before header
     endif
   endif
   return '=' " return previous fold level
 endfunction
}}}
        "#}
            .into(),
        vimwiki_header!("== Task Management ==")
            .into(),
        vimwiki_paragraph! {r#"
            Vimwiki makes it effortless to add tasks to any wiki page. Unfortunately,
            this means that your tasks get dispersed rather widely, especially if
            you're tracking action items from meeting notes in your diary. The snippets
            below make it easier to manage tasks in vimwiki without adding any additional
            plugins or relying on external task management tools.
        "#}
            .into(),
        vimwiki_header!("=== Find Incomplete Tasks ===")
            .into(),
        vimwiki_paragraph! {r#"
            The following will open a QuickFix window with incomplete tasks, but only those
            which are in a hyphenated (`-`) list. This is a simple way to filter only on
            tasks which are ready to be performed.
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
function! VimwikiFindIncompleteTasks()
  lvimgrep /- \[ \]/ %:p
  lopen
endfunction

function! VimwikiFindAllIncompleteTasks()
  VimwikiSearch /- \[ \]/
  lopen
endfunction

nmap <Leader>wa :call VimwikiFindAllIncompleteTasks()<CR>
nmap <Leader>wx :call VimwikiFindIncompleteTasks()<CR>
}}}
        "#}
            .into(),
        vimwiki_header!("== Encrypting Vimwiki pages ==")
            .into(),
        vimwiki_paragraph! {r#"
            If you want to encrypt singe pages of your wiki you can use [[https://github.com/jamessan/vim-gnupg|vim gnupg]] in
            conjunction with vimwiki. Add the following to your `vimrc`:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
let g:GPGFilePattern = '*.\(gpg\|asc\|pgp\)\(.wiki\)\='
}}}
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            Then you can create a link to a page in the following form: `[[link.asc]]`, the
            resulting file "link.asc.wiki" will be transparently encrypted by vim-gnupg.
            vim-gnupg will ask you to choose a key and gpg-agent will ask you to unlock the
            chosen key.
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            Note: If you use a different file-extension for your wikipages make sure to
            change the code above accordingly.
        "#}
            .into(),
        vimwiki_header!("== Cite entries from bibtex library ==")
            .into(),
        vimwiki_paragraph! {r#"
            Vimwiki has no support built in yet, but see [[https://github.com/vimwiki/vimwiki/issues/361|this issue]] for workarounds.
        "#}
            .into(),
        vimwiki_header!("== FAQ ==")
            .into(),
        vimwiki_header!("=== General ===")
            .into(),
        vimwiki_header!("==== How to change the folder of the wiki? ====")
            .into(),
        vimwiki_paragraph! {r#"
            You have to configure your wiki(s) in your vimrc, then you can configure among
            other the folder.
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
let g:vimwiki_list = [{'path': '~/mywiki/',
                      \ 'path_html': '~/mywiki_html'}]
}}}
        "#}
            .into(),
        vimwiki_header!("==== Can I start Vimwiki directly from shell? ====")
            .into(),
        vimwiki_paragraph! {r#"
            Yes:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{bash
$ vim -c VimwikiIndex
}}}
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            Opening the file of a wikipage also does the trick, that way you can open it
            with another than your main page. Example:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{bash
$ alias importantpage='vim vimwiki/importantpage.wiki'
$ importantpage
}}}
        "#}
            .into(),
        vimwiki_header!("==== Useful shell function for git integration and launch ====")
            .into(),
        vimwiki_paragraph! {r#"
            If you init your vimwiki directory as a git repo, and add the following function
            to your `.bashrc` or `.zshrc`, you can interact with the repo using the command
            `vimwiki git [commands]` from any directory:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{bash
vimwiki () {
    if [[ $# == 0 ]]
    then
        nvim +'VimwikiIndex'
    elif [[ $1 == 'git' ]]
    then
        git -C ~/vimwiki/ ${@:2}
    else
        echo 'Usage: vimwiki [git] [args ...]'
    fi
}
}}}
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            In addition, calling `vimwiki` without a git subcommand will automatically open
            the index.
        "#}
            .into(),
        vimwiki_header!("=== Markdown ===")
            .into(),
        vimwiki_header!("==== How do I use markdown syntax for my wikis? ====")
            .into(),
        vimwiki_paragraph! {r#"
            You have to configure your wiki(s) in your vimrc, then you can configure syntax
            and file extension. To set them to markdown and `.md` add the following
            configuration to you vimrc:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
let g:vimwiki_list = [{'path': '~/vimwiki/',
                      \ 'syntax': 'markdown', 'ext': '.md'}]
}}}
        "#}
            .into(),
        vimwiki_header!("==== Vimwiki considers every markdown-file as a wiki file ====")
            .into(),
        vimwiki_paragraph! {r#"
            Vimwiki has a feature called "Temporary Wikis", that will treat every file with
            configured file-extension as a wiki. To disable this feature add this to your vimrc:
        "#}
            .into(),
        vimwiki_code_block_raw! {r#"
{{{vim
let g:vimwiki_global_ext = 0
}}}
        "#}
            .into(),
        vimwiki_paragraph! {r#"
            Alternative you can set vimwiki to use markdown syntax but a different
            file-extension, like the default `.wiki`.
        "#}
            .into(),
        vimwiki_header!("== Got Other Great Ideas You'd Like to Share? ==")
            .into(),
        vimwiki_paragraph! {r#"
            If you have other snippets you find useful, please share them here on the wiki.
        "#}
            .into(),
    ];

    compare_page_elements(page.elements(), &expected);
}
