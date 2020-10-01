use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Blockquote {
    pub lines: Vec<String>,
}
