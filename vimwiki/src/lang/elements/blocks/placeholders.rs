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

impl Placeholder<'_> {
    pub fn to_borrowed(&self) -> Placeholder {
        use self::Cow::*;
        match self {
            Self::Title(ref x) => Placeholder::Title(Cow::Borrowed(match x {
                Borrowed(x) => *x,
                Owned(x) => x.as_str(),
            })),
            Self::NoHtml => Placeholder::NoHtml,
            Self::Template(ref x) => {
                Placeholder::Template(Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                }))
            }
            Self::Date(x) => Placeholder::Date(*x),
            Self::Other {
                ref name,
                ref value,
            } => Placeholder::Other {
                name: Cow::Borrowed(match name {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                }),
                value: Cow::Borrowed(match value {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                }),
            },
        }
    }

    pub fn into_owned(self) -> Placeholder<'static> {
        match self {
            Self::Title(x) => Placeholder::Title(Cow::from(x.into_owned())),
            Self::NoHtml => Placeholder::NoHtml,
            Self::Template(x) => {
                Placeholder::Template(Cow::from(x.into_owned()))
            }
            Self::Date(x) => Placeholder::Date(x),
            Self::Other { name, value } => Placeholder::Other {
                name: Cow::from(name.into_owned()),
                value: Cow::from(value.into_owned()),
            },
        }
    }
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
