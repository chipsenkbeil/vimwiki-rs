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
    #[clap(long = "wiki", number_of_values = 1)]
    pub wikis: Vec<WikiOpt>,

    /// Mode to run server
    #[clap(long, arg_enum, default_value = "http")]
    pub mode: ModeOpt,
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
    index: u32,
    name: Option<String>,
    path: PathBuf,
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
