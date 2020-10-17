use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct MathBlock<'a> {
    pub lines: Vec<Cow<'a, str>>,
    pub environment: Option<Cow<'a, str>>,
}

impl MathBlock<'_> {
    pub fn to_borrowed(&self) -> MathBlock {
        use self::Cow::*;

        MathBlock {
            lines: self
                .lines
                .iter()
                .map(|x| {
                    Cow::Borrowed(match x {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    })
                })
                .collect(),
            environment: self.environment.map(|x| {
                Cow::Borrowed(match &x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            }),
        }
    }

    pub fn into_owned(self) -> MathBlock<'static> {
        MathBlock {
            lines: self
                .lines
                .iter()
                .map(|x| Cow::from(x.into_owned()))
                .collect(),
            environment: self.environment.map(|x| Cow::from(x.into_owned())),
        }
    }
}
