/// Represents one or multiple root ids
pub trait Root: private::Sealed {
    #[doc(hidden)]
    fn to_vec(&self) -> Vec<usize>;
}

/// Represents a single root id
pub type SingleRoot = usize;

/// Represents multiple root ids
pub type MultiRoot = Vec<usize>;

impl Root for usize {
    fn to_vec(&self) -> Vec<usize> {
        vec![*self]
    }
}

impl Root for Vec<usize> {
    fn to_vec(&self) -> Vec<usize> {
        self.clone()
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for usize {}
    impl Sealed for Vec<usize> {}
}
