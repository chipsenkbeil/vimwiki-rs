use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreformattedText<'a> {
    pub lang: Option<Cow<'a, str>>,
    pub metadata: HashMap<Cow<'a, str>, Cow<'a, str>>,
    pub lines: Vec<Cow<'a, str>>,
}
