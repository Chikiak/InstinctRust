//! Species definition: a kind of creature with stable identity.
//! In F0-04 a species has no program yet (that comes in F0-06).

/// A kind of creature. Stores static traits; the program is added in F0-06.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Species {
    pub name: String,
}

impl Species {
    pub fn new(name: impl Into<String>) -> Self {
        Species { name: name.into() }
    }
}
