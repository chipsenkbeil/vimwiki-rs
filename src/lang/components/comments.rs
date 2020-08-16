use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineComment(String);

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MultiLineComment(Vec<String>);
