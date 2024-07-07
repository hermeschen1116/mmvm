use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Address(u16);

impl Address {
    pub fn new(addr_low: u8, addr_high: u8) -> Self {
        Self(u16::from_le_bytes([addr_low, addr_high]))
    }

    pub fn len() -> usize {
        2
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Address(address) = self;
        write!(f, "{:04x}", address)
    }
}
