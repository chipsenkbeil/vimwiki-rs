use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor,
    Clone,
    Debug,
    Display,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct CodeInline {
    pub code: String,
}

impl From<&str> for CodeInline {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}
