use clap::Clap;
use derive_more::{Display, Error};
use std::path::PathBuf;

#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub struct Opt {
    /// Activate debug mode
    #[clap(long)]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Wiki paths to load, monitor, and manipulate
    /// Format is index[:name]:path
    #[clap(long = "wiki", number_of_values = 1)]
    pub wikis: Vec<WikiOpt>,

    /// Mode to run server (http = web; stdin = read input from stdin and reply on stdout)
    #[clap(long, arg_enum, default_value = "http")]
    pub mode: ModeOpt,

    /// Host/IP address of server
    #[clap(long, default_value = "localhost")]
    pub host: String,

    /// Port of the server
    #[clap(long, default_value = "8000")]
    pub port: u16,

    /// Extensions for wiki files to parse
    #[clap(long = "ext", number_of_values = 1, default_value = "wiki")]
    pub exts: Vec<String>,
}

/// Represents the mode to run the server (input from stdin or HTTP)
#[derive(Clap, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModeOpt {
    Stdin,
    Http,
}

/// Represents input information about a wiki
#[derive(Debug)]
pub struct WikiOpt {
    pub index: u32,
    pub name: Option<String>,
    pub path: PathBuf,
}

impl std::fmt::Display for WikiOpt {
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
pub enum ParseWikiOptError {
    InvalidPath,
    InvalidIndex,
    InvalidName,
    InvalidInput,
}

impl std::str::FromStr for WikiOpt {
    type Err = ParseWikiOptError;

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