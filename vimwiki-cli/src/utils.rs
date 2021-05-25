use crate::VimVar;
use log::*;
use vimwiki::HtmlWikiConfig;

/// Loads g:vimwiki_list from vim/neovim and then attempts to convert it into
/// a structured html wiki config
pub fn load_vimwiki_list() -> std::io::Result<Vec<HtmlWikiConfig>> {
    trace!("load_vimwiki_list()");

    let vimwiki_list_json = VimVar::get_global("vimwiki_list", false)?;
    trace!("g:vimwiki_list == {:?}", vimwiki_list_json);

    if let Some(json) = vimwiki_list_json {
        serde_json::from_value(json).map_err(Into::into)
    } else {
        Ok(Vec::new())
    }
}
