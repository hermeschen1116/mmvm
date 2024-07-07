use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IC(u16);

impl IC {
    pub fn new(instruction_pointer: u16) -> Self {
        Self(instruction_pointer)
    }
}

impl Add<u16> for IC {
    type Output = Self;

    fn add(self, other: u16) -> Self {
        let IC(instruction_pointer) = self;
        if let Some(instruction_pointer) = instruction_pointer.checked_add(other) {
            IC::new(instruction_pointer)
        } else {
            panic!("overflow occur")
        }
    }
}

impl Add<i16> for IC {
    type Output = Self;

    fn add(self, other: i16) -> Self {
        let IC(instruction_pointer) = self;
        if let Some(instruction_pointer) = instruction_pointer.checked_add_signed(other) {
            IC::new(instruction_pointer)
        } else {
            panic!("overflow occur")
        }
    }
}

impl Display for IC {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &IC(ic) = self;
        write!(f, "{:04x}", ic)
    }
}
