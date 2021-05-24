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
    #[structopt(long)]
    pub log_timestamp: Option<stderrlog::Timestamp>,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Convert(ConvertSubcommand),
    Serve(ServeSubcommand),
}

/// Convert vimwiki into something else
#[derive(Debug, StructOpt)]
pub struct ConvertSubcommand {
    /// Write output to FILE instead of stdout; if processing multiple files,
    /// can use `{}` as a filler for the file name such as `{}.html`
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Path to config file for output (otherwise uses default settings)
    #[structopt(long)]
    pub config: Option<PathBuf>,

    /// Extensions of files to parse when loading from wikis or arbitrary
    /// directories
    #[structopt(long = "ext", default_value = "wiki")]
    pub extensions: Vec<String>,

    /// If provided, will fail immediately when encountering an error instead
    /// of continuing
    #[structopt(long)]
    pub fail_fast: bool,

    /// If provided, will attempt to load all wikis and generate output
    #[structopt(long)]
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
}
