use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Math {
    Inline(MathInline),
    BlockDisplay(MathBlockDisplay),
    BlockEnvironment(MathBlockEnvironment),
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MathInline {
    formula: String,
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MathBlockDisplay {
    lines: Vec<String>,
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MathBlockEnvironment {
    environment: String,
    lines: Vec<String>,
}
