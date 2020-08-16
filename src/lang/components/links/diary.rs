use super::{WithAnchor, WithDescription};
use chrono::naive::NaiveDate;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// Represents a link to an entry in the diary wiki
#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiaryLink {
    date: NaiveDate,
    description: Option<String>,
    anchor: Option<String>,
}

impl DiaryLink {
    pub fn date(&self) -> &NaiveDate {
        &self.date
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

impl WithDescription for DiaryLink {
    fn with_description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
        self
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl WithAnchor for DiaryLink {
    fn with_anchor(&mut self, anchor: String) -> &mut Self {
        self.anchor = Some(anchor);
        self
    }

    fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }
}
