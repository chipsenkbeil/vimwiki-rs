mod vim;
use vim::VimVar;

use std::path::PathBuf;
use structopt::StructOpt;
use vimwiki::*;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Path to html config file for html output (otherwise uses default settings)
    #[structopt(long)]
    html_config: Option<PathBuf>,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let mut html_config = if let Some(path) = opt.html_config {
        let config_string = std::fs::read_to_string(path)
            .expect("Failed to load html config file");
        toml::from_str(config_string.as_str())
            .expect("Failed to parse html config file")
    } else {
        HtmlConfig::default()
    };

    // If html config has no wikis, attempt to load wikis from vim
    if html_config.wikis.is_empty() {
        // We attempt to load and parse our wiki content now, and if it fails
        // then we report over stderr and continue
        match load_vimwiki_list() {
            Ok(wikis) => html_config.wikis = wikis,
            Err(x) => {
                eprintln!("Failed to load vimwiki_list from vim/neovim: {}", x)
            }
        }
    }

    println!("Config: {:?}", html_config);
}

/// Loads g:vimwiki_list from vim/neovim and then attempts to convert it into
/// a structured html wiki config
fn load_vimwiki_list() -> std::io::Result<Vec<HtmlWikiConfig>> {
    let vimwiki_list_json = VimVar::get_global("vimwiki_list")?;
    serde_json::from_value(vimwiki_list_json).map_err(Into::into)
}
