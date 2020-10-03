use super::{Anchor, Description};
use chrono::naive::NaiveDate;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// Represents a link to an entry in the diary wiki
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct DiaryLink {
    pub date: NaiveDate,
    pub description: Option<Description>,
    pub anchor: Option<Anchor>,
}

impl fmt::Display for DiaryLink {
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

impl From<NaiveDate> for DiaryLink {
    fn from(date: NaiveDate) -> Self {
        Self::new(date, None, None)
    }
}

impl TryFrom<&str> for DiaryLink {
    type Error = chrono::format::ParseError;

    fn try_from(str_date: &str) -> Result<Self, Self::Error> {
        let date = NaiveDate::parse_from_str(str_date, "%Y-%m-%d")?;
        Ok(Self::from(date))
    }
}
