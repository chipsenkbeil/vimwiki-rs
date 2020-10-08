use super::Element;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct MathBlock {
    pub lines: Vec<String>,
    pub environment: Option<String>,
}

impl Element for MathBlock {}
