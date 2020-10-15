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

impl<'a> Placeholder<'a> {
    pub fn title_from_str(title: &'a str) -> Self {
        Self::Title(Cow::from(title))
    }

    pub fn title_from_string(title: String) -> Self {
        Self::Title(Cow::from(title))
    }

    pub fn template_from_str(template: &'a str) -> Self {
        Self::Template(Cow::from(template))
    }

    pub fn template_from_string(template: String) -> Self {
        Self::Template(Cow::from(template))
    }

    pub fn other_from_str(name: &'a str, value: &'a str) -> Self {
        Self::Other {
            name: Cow::from(name),
            value: Cow::from(value),
        }
    }

    pub fn other_from_string(name: String, value: String) -> Self {
        Self::Other {
            name: Cow::from(name),
            value: Cow::from(value),
        }
    }
}
