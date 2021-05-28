use crate::{utils, CommonOpt, IndexOrName, PrintSubcommand, PrintType};
use log::*;
use std::io;
use vimwiki::HtmlConfig;

pub fn print(cmd: PrintSubcommand, _opt: CommonOpt) -> io::Result<()> {
    let config: HtmlConfig = if let Some(path) = cmd.config {
        let config_string = std::fs::read_to_string(path)?;
        toml::from_str(config_string.as_str())?
    } else {
        HtmlConfig::default()
    };

    match cmd.ty {
        PrintType::Wiki { all, merge, target } => {
            print_wiki(config, all, merge, target)
        }
    }
}

fn print_wiki(
    config: HtmlConfig,
    all: bool,
    merge: bool,
    target: Option<IndexOrName>,
) -> io::Result<()> {
    let mut wikis = Vec::new();

    // If we are merging or have no wikis, we want to try to load from vim/neovim
    if merge || config.wikis.is_empty() {
        match utils::load_vimwiki_list() {
            Ok(list) => wikis.extend(list),
            Err(x) => error!("Failed to load vim/neovim wikis: {}", x),
        }
    }

    // Next, add the config file wikis
    wikis.extend(config.wikis);

    // For each wiki that passes our criteria, print it out
    for (idx, wiki) in wikis.into_iter().enumerate().filter(|(idx, wiki)| {
        all || target.as_ref().map_or(false, |target| target == idx)
            || target
                .as_ref()
                .zip(wiki.name.as_ref())
                .map_or(false, |(target, name)| target == name)
    }) {
        println!("wiki {}", idx);
        println!("| path = \"{}\"", wiki.path.to_string_lossy());
        println!("| path_html = \"{}\"", wiki.path_html.to_string_lossy());

        if let Some(name) = wiki.name.as_deref() {
            println!("| name = \"{}\"", name);
        } else {
            println!("| name = <NONE>");
        }

        println!("| css_name = \"{}\"", wiki.css_name);
        println!(
            "| diary_rel_path = \"{}\"",
            wiki.diary_rel_path.to_string_lossy()
        );
        println!();
    }

    Ok(())
}
