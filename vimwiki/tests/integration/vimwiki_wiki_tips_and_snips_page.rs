use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_elements},
};
use std::convert::TryInto;
use vimwiki::{elements::*, RawStr, Region, LE};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LE<Page> =
        RawStr::Vimwiki(&VimwikiFile::VimwikiWikiTipsAndSnips.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        vimwiki_header!("= Tips and Snips =")
            .take_with_region(Region::from((1, 1, 1, 19)))
            .into(),
        blank_line()
            .take_with_region(Region::from((2, 1, 2, 1))),
        vimwiki_paragraph! {r#"
            Vimwiki cannot be all things to all users so here are some tips and code
            snippets you may find useful for customizing Vimwiki to your liking.
        "#}
            .take_with_region(Region::from((3, 1, 4, 69)))
            .into(),
        blank_line()
            .take_with_region(Region::from((5, 1, 5, 1))),
        vimwiki_header!("== Cheat Sheet ==")
            .take_with_region(Region::from((6, 1, 6, 18)))
            .into(),
        vimwiki_paragraph! {r#"
            There are several cheat sheets for Vimwiki:
        "#}
            .take_with_region(Region::from((7, 1, 7, 44)))
            .into(),
        vimwiki_list! {r#"
            - [[http://thedarnedestthing.com/vimwiki%20cheatsheet|The Darnedest Thing - Vimwiki Cheatsheet]]
            - [[https://gist.github.com/drkarl/4c503bccb62558dc85e8b1bc0f29e9cb|Short Cheatsheet]]
            - [[https://dokk.org/library/Vimwiki_1.1.1_Quick_Reference_(Pospichal_2001)|PDF Cheatsheet incl. Syntax]]
        "#}
            .take_with_region(Region::from((8, 1, 10, 106)))
            .into(),
        blank_line()
            .take_with_region(Region::from((11, 1, 11, 1))),
        vimwiki_header!("== Custom folding ==")
            .take_with_region(Region::from((12, 1, 12, 21)))
            .into(),
        blank_line()
            .take_with_region(Region::from((13, 1, 13, 1))),
        vimwiki_paragraph! {r#"
            Vimwiki has simple folding folding methods built in but lets you easily
            customize folds with the `g:vimwiki_folding` option. For example, if you prefer
            that the last blank line before a heading not get folded, add this to your
            `.vimrc` file:
        "#}
            .take_with_region(Region::from((14, 1, 17, 15)))
            .into(),
        blank_line()
            .take_with_region(Region::from((18, 1, 18, 1))),
        vimwiki_paragraph! {r#"
            `let g:vimwiki_folding = 'custom'`
        "#}
            .take_with_region(Region::from((19, 1, 19, 35)))
            .into(),
        blank_line()
            .take_with_region(Region::from((20, 1, 20, 1))),
        vimwiki_paragraph! {r#"
            Then add the following to the `ftplugin/vimwiki.vim` plugin in your `.vim`
            configuration folder (create this file if it doesn't already exist):
        "#}
            .take_with_region(Region::from((21, 1, 22, 69)))
            .into(),
        blank_line()
            .take_with_region(Region::from((23, 1, 23, 1))),
        vimwiki_preformatted_text_raw! {r#"
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
            .take_with_region(Region::from((24, 1, 44, 4)))
            .into(),
        blank_line()
            .take_with_region(Region::from((45, 1, 45, 1))),
    ];

    compare_page_elements(&page.elements, &expected);
}
