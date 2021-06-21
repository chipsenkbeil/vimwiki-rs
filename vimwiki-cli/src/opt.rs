use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;
use structopt::StructOpt;

lazy_static! {
    static ref DEFAULT_CACHE_DIR: String =
        ProjectDirs::from("rs", "vimwiki", "vimwiki-cli")
            .map(|dir| dir.cache_dir().to_string_lossy().to_string())
            .unwrap_or_default();
}

/// Tooling to convert and manipulation vimwiki files and wikis
#[derive(Debug, StructOpt)]
#[structopt(name = "vimwiki")]
pub struct Opt {
    #[structopt(flatten)]
    pub common: CommonOpt,

    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
pub struct CommonOpt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences), global = true)]
    pub verbose: usize,

    /// Quiet mode
    #[structopt(short, long, global = true)]
    pub quiet: bool,

    /// Timestamp for logging (sec, ms, ns, none)
    #[structopt(short, long, global = true)]
    pub timestamp: Option<stderrlog::Timestamp>,

    /// Directory where cache information is stored
    #[structopt(long, default_value = &DEFAULT_CACHE_DIR, global = true)]
    pub cache: PathBuf,

    /// If specified, no cache will be used
    #[structopt(long, global = true)]
    pub no_cache: bool,

    /// If specified, cache directory will not be pruned of old files
    #[structopt(long, global = true)]
    pub no_prune_cache: bool,

    /// Path to config file
    #[structopt(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// If specified, will attempt to merge wikis loaded from vim/neovim
    /// with wikis defined via a config file if accessible. Wikis from
    /// vim/neovim will be first such that their indexes align with those
    /// defined in vimscript with the config file wikis being added after
    ///
    /// If not specified, then vim/neovim wikis are only loaded if there
    /// is no config file or the config file has no wikis defined
    #[structopt(long, global = true)]
    pub merge: bool,

    /// Specifies specific wikis to include by index or name; if none are
    /// provided, then all available wikis are converted
    #[structopt(long, global = true)]
    pub include: Vec<IndexOrName>,
}

impl CommonOpt {
    /// Filter for wikis to process, defaulting to every wiki unless given a
    /// filter of wikis to include
    pub fn filter_by_wiki_idx_and_name(
        &self,
        idx: usize,
        name: Option<&str>,
    ) -> bool {
        self.include.is_empty()
            || self.include.iter().any(|f| f.matches_either(idx, name))
    }
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Convert(ConvertSubcommand),
    Format(FormatSubcommand),
    Inspect(InspectSubcommand),
    Serve(ServeSubcommand),
}

impl Subcommand {
    /// Returns extra paths to process
    pub fn extra_paths(&self) -> &[PathBuf] {
        match self {
            Self::Convert(x) => &x.extra_paths,
            Self::Format(x) => &x.paths,
            Self::Inspect(x) => &x.extra_paths,
            Self::Serve(x) => &x.extra_paths,
        }
    }
}

/// Convert vimwiki into something else
#[derive(Debug, StructOpt)]
pub struct ConvertSubcommand {
    /// Write output to stdout instead of file system
    #[structopt(long)]
    pub stdout: bool,

    /// If provided, will include vimwiki's style.css file at the root of
    /// the wiki's output directory
    #[structopt(long)]
    pub include_vimwiki_css: bool,

    /// Additional standalone files (or directories) to process
    #[structopt(name = "PATH", parse(from_os_str))]
    pub extra_paths: Vec<PathBuf>,
}

/// Format vimwiki files following a configuration
#[derive(Debug, StructOpt)]
pub struct FormatSubcommand {
    /// Apply format inline, overwritting each file
    #[structopt(short, long)]
    pub inline: bool,

    /// Extensions to use when searching through directories
    #[structopt(long = "ext", default_value = "wiki")]
    pub extensions: Vec<String>,

    /// Files (or directories) to process
    #[structopt(name = "PATH", parse(from_os_str))]
    pub paths: Vec<PathBuf>,
}

/// Convert vimwiki into something else and serve it via http
#[derive(Debug, StructOpt)]
pub struct ServeSubcommand {
    /// Web port to listen on to serve requests
    #[structopt(short, long, default_value = "8080")]
    pub port: usize,

    /// If provided, will include vimwiki's styles.css file at the root of
    /// the output directory
    #[structopt(long)]
    pub include_styles_css: bool,

    /// Additional standalone files (or directories) to process
    #[structopt(name = "PATH", parse(from_os_str))]
    pub extra_paths: Vec<PathBuf>,
}

/// Inspect information that is available
#[derive(Debug, StructOpt)]
pub struct InspectSubcommand {
    /// Writes to output file instead of stdout
    #[structopt(short, long)]
    pub output: Option<PathBuf>,

    /// JSON path to use for inspection
    #[structopt(name = "JSONPATH")]
    pub json_path: String,

    /// Additional standalone files (or directories) to process
    #[structopt(name = "PATH", parse(from_os_str))]
    pub extra_paths: Vec<PathBuf>,
}

/// Represents either a wiki index or a wiki name
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexOrName {
    Index(usize),
    Name(String),
}

impl IndexOrName {
    /// Returns true if the index matches either the index or name provided
    pub fn matches_either<'a, N: Into<Option<&'a str>>>(
        &self,
        index: usize,
        name: N,
    ) -> bool {
        self == &index || name.into().map_or(false, |name| self == name)
    }
}

impl PartialEq<usize> for IndexOrName {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Self::Index(x) => x == other,
            _ => false,
        }
    }
}

impl PartialEq<String> for IndexOrName {
    fn eq(&self, other: &String) -> bool {
        match self {
            Self::Name(x) => x == other,
            _ => false,
        }
    }
}

impl PartialEq<str> for IndexOrName {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Name(x) => x == other,
            _ => false,
        }
    }
}

impl std::str::FromStr for IndexOrName {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(idx) => Ok(Self::Index(idx)),
            Err(_) => Ok(Self::Name(s.to_string())),
        }
    }
}
