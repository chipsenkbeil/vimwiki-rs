use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Placeholder<'a> {
    Title(Cow<'a, str>),
    NoHtml,
    Template(Cow<'a, str>),
    Date(NaiveDate),
    Other {
        name: Cow<'a, str>,
        value: Cow<'a, str>,
    },
}
