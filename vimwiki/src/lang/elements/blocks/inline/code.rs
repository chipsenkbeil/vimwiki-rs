use super::Element;
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

impl Element for CodeInline {}
