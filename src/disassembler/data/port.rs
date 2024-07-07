use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Port(u8);

impl Port {
    pub fn new(byte: u8) -> Self {
        Self(u8::from(byte))
    }

    pub fn len() -> usize {
        1
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Port(port) = self;
        write!(f, "{:x}", port)
    }
}
