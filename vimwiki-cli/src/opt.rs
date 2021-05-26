use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: usize,

    /// Quiet mode
    #[structopt(short, long)]
    pub quiet: bool,

    /// Timestamp for logging (sec, ms, ns, none)
    #[structopt(short, long)]
    pub timestamp: Option<stderrlog::Timestamp>,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Convert(ConvertSubcommand),
    Serve(ServeSubcommand),
    Print(PrintSubcommand),
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

    /// Path to config file for output (otherwise uses default settings)
    #[structopt(short, long)]
    pub config: Option<PathBuf>,

    /// Extensions of files to parse when loading from wikis or arbitrary
    /// directories
    #[structopt(short, long = "ext", default_value = "wiki")]
    pub extensions: Vec<String>,

    /// If provided, will fail immediately when encountering an error instead
    /// of continuing
    #[structopt(long)]
    pub fail_fast: bool,

    /// If provided, will attempt to load all wikis and generate output
    #[structopt(short, long)]
    pub all: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub files: Vec<PathBuf>,
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

    /// Path to config file for output (otherwise uses default settings)
    #[structopt(short, long)]
    pub config: Option<PathBuf>,

    /// Extensions of files to parse when loading from wikis or arbitrary
    /// directories
    #[structopt(short, long = "ext", default_value = "wiki")]
    pub extensions: Vec<String>,

    /// If provided, will attempt to load all wikis and generate output
    #[structopt(short, long)]
    pub all: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub files: Vec<PathBuf>,
}

/// Print out information that is available
#[derive(Debug, StructOpt)]
pub struct PrintSubcommand {
    /// Path to config file to inspect for printing
    #[structopt(long)]
    pub config: Option<PathBuf>,

    #[structopt(subcommand)]
    pub ty: PrintType,
}

#[derive(Clone, Debug, StructOpt)]
pub enum PrintType {
    /// Print information about one or more wikis
    Wiki {
        /// Print all loaded wikis
        #[structopt(short, long)]
        all: bool,

        /// If specified, will attempt to merge wikis loaded from vim/neovim
        /// with wikis defined via a config file if accessible. Wikis from
        /// vim/neovim will be first such that their indexes align with those
        /// defined in vimscript with the config file wikis being added after
        ///
        /// If not specified, then vim/neovim wikis are only loaded if there
        /// is no config file or the config file has no wikis defined
        #[structopt(short, long)]
        merge: bool,

        /// Print wiki with specific index or name
        #[structopt(name = "TARGET")]
        target: Option<IndexOrName>,
    },
}

/// Represents either a wiki index or a wiki name
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexOrName {
    Index(usize),
    Name(String),
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
