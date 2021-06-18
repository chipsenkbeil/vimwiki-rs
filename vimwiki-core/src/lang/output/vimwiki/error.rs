use derive_more::{Display, Error, From};

pub type VimwikiOutputResult = Result<(), VimwikiOutputError>;

#[derive(Debug, From, Display, Error)]
pub enum VimwikiOutputError {
    Fmt {
        #[error(source)]
        source: std::fmt::Error,
    },
}
