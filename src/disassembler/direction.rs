#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    FromReg = 0b0,
    ToReg = 0b1,
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            0b0 => Direction::FromReg,
            0b1 => Direction::ToReg,
            _ => panic!("Value should be 0b0 or 0b1"),
        }
    }
}
