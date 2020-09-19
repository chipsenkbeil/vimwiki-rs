use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreformattedText {
    pub lang: Option<String>,
    pub metadata: HashMap<String, String>,
    pub lines: Vec<String>,
}
