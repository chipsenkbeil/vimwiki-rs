use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreformattedText {
    metadata: HashMap<String, String>,
    lines: Vec<String>,
}
