use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

lazy_static! {
    static ref DEFAULT_CACHE_DIR: String =
        ProjectDirs::from("rs", "vimwiki", "vimwiki-server")
            .map(|dir| dir.cache_dir().to_string_lossy().to_string())
            .unwrap_or_default();
}

#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Specify a directory to store log output as files rather than stdout/stderr
    #[structopt(long)]
    pub log_dir: Option<PathBuf>,

    /// Mode to run server (http = web; stdin = read input from stdin and reply on stdout)
    #[structopt(long, possible_values = Mode::VARIANTS, case_insensitive = true, default_value = "http")]
    pub mode: Mode,

    /// Host/IP address of server in http mode
    #[structopt(long, default_value = "localhost")]
    pub host: String,

    /// Port of the server in http mode
    #[structopt(long, default_value = "8000")]
    pub port: u16,

    /// Directory where cache information for use with server will be stored
    #[structopt(long, default_value = &DEFAULT_CACHE_DIR)]
    pub cache_dir: PathBuf,

    /// Path to config file for wiki definitions
    #[structopt(long)]
    pub config: Option<PathBuf>,

    /// If specified, will attempt to merge wikis loaded from vim/neovim
    /// with wikis defined via a config file if accessible. Wikis from
    /// vim/neovim will be first such that their indexes align with those
    /// defined in vimscript with the config file wikis being added after
    ///
    /// If not specified, then vim/neovim wikis are only loaded if there
    /// is no config file or the config file has no wikis defined
    #[structopt(short, long)]
    pub merge: bool,
}

impl Opt {
    pub fn load() -> Self {
        Self::from_args()
    }

    /// The level to use for logging throughout the server
    pub fn log_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

/// Represents the mode to run the server (input from stdin or HTTP)
#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
pub enum Mode {
    Stdin,
    Http,
}
