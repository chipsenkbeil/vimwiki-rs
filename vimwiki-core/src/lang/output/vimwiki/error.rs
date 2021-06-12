use derive_more::{Display, Error};

pub type VimwikiOutputResult = Result<(), VimwikiOutputError>;

#[derive(Debug, Display, Error)]
pub enum VimwikiOutputError {
    Fmt {
        #[error(source)]
        source: std::fmt::Error,
    },
}
