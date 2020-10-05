use clap::Clap;
use derive_more::{Display, Error};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref DEFAULT_CACHE_DIR: String =
        ProjectDirs::from("com", "chipsenkbeil", "vimwiki_server")
            .map(|dir| dir.cache_dir().to_string_lossy().to_string())
            .unwrap_or_default();
}

#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Config {
    /// Activate debug mode
    #[clap(long)]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Wiki paths to load, monitor, and manipulate
    /// Format is index[:name]:path
    #[clap(long = "wiki", number_of_values = 1)]
    pub wikis: Vec<WikiConfig>,

    /// Mode to run server (http = web; stdin = read input from stdin and reply on stdout)
    #[clap(long, arg_enum, default_value = "http")]
    pub mode: Mode,

    /// Host/IP address of server
    #[clap(long, default_value = "localhost")]
    pub host: String,

    /// Port of the server
    #[clap(long, default_value = "8000")]
    pub port: u16,

    /// Extensions for wiki files to parse
    #[clap(long = "ext", number_of_values = 1, default_value = "wiki")]
    pub exts: Vec<String>,

    /// Directory where cache information for use with server will be stored
    #[clap(long, default_value = &DEFAULT_CACHE_DIR)]
    pub cache_dir: PathBuf,
}

/// Represents the mode to run the server (input from stdin or HTTP)
#[derive(Clap, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Stdin,
    Http,
}

/// Represents input information about a wiki
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WikiConfig {
    pub index: u32,
    pub name: Option<String>,
    pub path: PathBuf,
}

impl std::fmt::Display for WikiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.index)?;

        if let Some(name) = self.name.as_ref() {
            write!(f, "/{}", name)?;
        }

        write!(f, ":{}", self.path.to_string_lossy())?;

        Ok(())
    }
}

/// Represents parsing errors that can occur for a wiki opt
#[derive(Debug, Display, Error)]
pub enum ParseWikiConfigError {
    InvalidPath,
    InvalidIndex,
    InvalidName,
    InvalidInput,
}

impl std::str::FromStr for WikiConfig {
    type Err = ParseWikiConfigError;

    /// Parse input in form of <index>[:<name>]:path
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        let parts_len: usize = parts.len();
        if !(2..=3).contains(&parts_len) {
            return Err(Self::Err::InvalidInput);
        }

        let index = parts[0]
            .parse::<u32>()
            .map_err(|_| Self::Err::InvalidIndex)?;

        let instance = if parts.len() == 2 {
            Self {
                index,
                name: None,
                path: PathBuf::from(parts[1]),
            }
        } else {
            Self {
                index,
                name: Some(parts[1].to_string()),
                path: PathBuf::from(parts[2]),
            }
        };

        // If name is not none, but is empty, return an error
        if instance
            .name
            .as_ref()
            .map(|x| x.is_empty())
            .unwrap_or_default()
        {
            return Err(Self::Err::InvalidName);
        }

        // If path does not exist, return an error
        if !instance.path.exists() {
            return Err(Self::Err::InvalidPath);
        }

        Ok(instance)
    }
}
