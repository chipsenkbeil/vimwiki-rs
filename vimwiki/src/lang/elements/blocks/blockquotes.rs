use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Blockquote<'a> {
    pub lines: Vec<Cow<'a, str>>,
}

impl Blockquote<'_> {
    pub fn to_borrowed(&self) -> Blockquote {
        use self::Cow::*;

        let lines = self
            .lines
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect();

        Blockquote { lines }
    }

    pub fn into_owned(self) -> Blockquote<'static> {
        let lines = self
            .lines
            .iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect();

        Blockquote { lines }
    }
}
