use crate::VimVar;
use log::*;
use std::{io, path::Path};
use vimwiki::{HtmlConfig, HtmlWikiConfig};

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

/// Attempts to load an html config from a file, attempting to load wikis from
/// vim/neovim if no wikis are defined or if merge = true
pub fn load_html_config<'a, I: Into<Option<&'a Path>>>(
    path: I,
    merge: bool,
) -> io::Result<HtmlConfig> {
    let maybe_path = path.into();
    trace!("load_html_config(path = {:?})", maybe_path);

    let mut html_config: HtmlConfig = if let Some(path) = maybe_path {
        let config_string = std::fs::read_to_string(path)?;
        toml::from_str(config_string.as_str())?
    } else {
        HtmlConfig::default()
    };

    // Attempt to load wikis from vim if html config has no wikis or if
    // we are explicitly told to merge
    if html_config.wikis.is_empty() || merge {
        // We attempt to load and parse our wiki content now, and if it fails
        // then we report over stderr and continue
        match load_vimwiki_list() {
            Ok(wikis) => html_config.wikis.extend(wikis),
            Err(x) => {
                error!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    Ok(html_config)
}
