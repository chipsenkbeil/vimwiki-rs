use crate::CommonOpt;
use log::*;
use std::{io, path::PathBuf};
use vimvar::VimVar;
use vimwiki::{HtmlConfig, HtmlWikiConfig};

/// Attempts to load an html config from a file, attempting to load wikis from
/// vim/neovim if no wikis are defined or if merge = true
pub fn load_html_config(
    opt: &CommonOpt,
    extra_paths: &[PathBuf],
) -> io::Result<HtmlConfig> {
    let CommonOpt {
        config,
        merge,
        include,
        ..
    } = opt;

    trace!(
        "load_html_config(path = {:?}, include = {:?}, merge = {}, extra_paths = {:?})",
        config,
        include,
        merge,
        extra_paths
    );

    let mut config: HtmlConfig = if let Some(path) = config {
        let config_string = std::fs::read_to_string(path)?;
        toml::from_str(config_string.as_str())?
    } else {
        HtmlConfig::default()
    };

    // Attempt to load wikis from vim if html config has no wikis or if
    // we are explicitly told to merge
    if config.wikis.is_empty() || *merge {
        // We attempt to load and parse our wiki content now, and if it fails
        // then we report over stderr and continue
        match load_vimwiki_list() {
            Ok(wikis) => config.wikis.extend(wikis),
            Err(x) => {
                error!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    // Add temporary wikis for standalone directories and files we are given
    for path in extra_paths.iter() {
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(x) => {
                error!("{:?} failed to canonicalize: {}", path, x);
                return Err(x);
            }
        };
        debug!("Creating temporary wiki for {:?}", path);
        if path.is_dir() {
            config.wikis.push(HtmlWikiConfig {
                path: path.to_path_buf(),
                ..Default::default()
            });
        } else if let Some(parent) = path.parent() {
            config.wikis.push(HtmlWikiConfig {
                path: parent.to_path_buf(),
                ..Default::default()
            });
        }
    }

    // Finally, filter out wikis based on include logic
    config.wikis = config
        .wikis
        .into_iter()
        .enumerate()
        .filter(|(idx, wiki)| {
            opt.filter_by_wiki_idx_and_name(*idx, wiki.name.as_deref())
        })
        .map(|(_, wiki)| wiki)
        .collect();

    Ok(config)
}

/// Loads g:vimwiki_list from vim/neovim and then attempts to convert it into
/// a structured html wiki config
fn load_vimwiki_list() -> std::io::Result<Vec<HtmlWikiConfig>> {
    trace!("load_vimwiki_list()");

    let vimwiki_list_json = VimVar::load_global_var("vimwiki_list", false)?;
    trace!("g:vimwiki_list == {:?}", vimwiki_list_json);

    if let Some(json) = vimwiki_list_json {
        serde_json::from_value(json).map_err(Into::into)
    } else {
        Ok(Vec::new())
    }
}
