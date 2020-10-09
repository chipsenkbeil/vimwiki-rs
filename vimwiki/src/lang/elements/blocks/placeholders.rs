use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Placeholder {
    Title(String),
    NoHtml,
    Template(String),
    Date(NaiveDate),
    Other { name: String, value: String },
}
