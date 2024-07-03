use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Displacement {
    Word(u16),
    Byte(u8),
}

impl Display for Displacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Displacement::Word(displacement) if displacement != 0x0 => {
                write!(f, "{:x}", displacement)
            }
            &Displacement::Byte(displacement) if displacement != 0b0 => {
                write!(f, "{:x}", displacement)
            }
            _ => write!(f, ""),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    Word(u16),
    Byte(u8),
}

impl Display for Immediate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Immediate::Word(immediate) => {
                write!(f, "{:04x}", immediate)
            }
            &Immediate::Byte(immediate) => {
                write!(f, "{:x}", immediate)
            }
        }
    }
}
