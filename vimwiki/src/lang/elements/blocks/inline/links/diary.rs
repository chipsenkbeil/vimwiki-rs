use super::{Anchor, Description};
use chrono::naive::NaiveDate;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// Represents a link to an entry in the diary wiki
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct DiaryLink<'a> {
    pub date: NaiveDate,
    pub description: Option<Description<'a>>,
    pub anchor: Option<Anchor<'a>>,
}

impl DiaryLink<'_> {
    pub fn to_borrowed(&self) -> DiaryLink {
        let date = self.date;
        let description =
            self.description.as_ref().map(Description::to_borrowed);
        let anchor = self.anchor.as_ref().map(Anchor::to_borrowed);

        DiaryLink {
            date,
            description,
            anchor,
        }
    }

    pub fn into_owned(self) -> DiaryLink<'static> {
        let date = self.date;
        let description = self.description.map(Description::into_owned);
        let anchor = self.anchor.map(Anchor::into_owned);

        DiaryLink {
            date,
            description,
            anchor,
        }
    }
}

impl<'a> fmt::Display for DiaryLink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = self.description.as_ref() {
            write!(f, "{}", desc)
        } else {
            write!(f, "{}", self.date)?;
            if let Some(anchor) = self.anchor.as_ref() {
                write!(f, "{}", anchor)?;
            }
            Ok(())
        }
    }
}

impl From<NaiveDate> for DiaryLink<'static> {
    fn from(date: NaiveDate) -> Self {
        Self::new(date, None, None)
    }
}

impl TryFrom<&str> for DiaryLink<'static> {
    type Error = chrono::format::ParseError;

    fn try_from(str_date: &str) -> Result<Self, Self::Error> {
        let date = NaiveDate::parse_from_str(str_date, "%Y-%m-%d")?;
        Ok(Self::from(date))
    }
}
