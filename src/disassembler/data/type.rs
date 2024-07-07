use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Type(u8);

impl Type {
    pub fn new(byte: u8) -> Self {
        Self(u8::from(byte))
    }

    pub fn len() -> usize {
        1
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Type(port) = self;
        write!(f, "{:x}", port)
    }
}
