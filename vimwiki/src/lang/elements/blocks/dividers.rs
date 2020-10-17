use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Divider;

impl Divider {
    pub fn as_borrowed(&self) -> Divider {
        *self
    }

    pub fn into_owned(self) -> Divider {
        self
    }
}
