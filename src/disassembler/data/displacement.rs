use std::cmp;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Displacement {
    Word(i16),
    Byte(i8),
    Zero,
}

impl From<u8> for Displacement {
    fn from(value: u8) -> Self {
        Displacement::Byte(value as i8)
    }
}

impl From<&[u8]> for Displacement {
    fn from(value: &[u8]) -> Self {
        let disp = u16::from_le_bytes(value.try_into().unwrap());
        Displacement::Word(disp as i16)
    }
}

impl Display for Displacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Displacement::Word(disp) => match disp.cmp(&0i16) {
                cmp::Ordering::Less => write!(f, "-{:x}", !(disp - 0b1)),
                cmp::Ordering::Equal => write!(f, ""),
                cmp::Ordering::Greater => write!(f, "+{:x}", disp),
            },
            &Displacement::Byte(disp) => match disp.cmp(&0i8) {
                cmp::Ordering::Less => write!(f, "-{:x}", !(disp - 0b1)),
                cmp::Ordering::Equal => write!(f, ""),
                cmp::Ordering::Greater => write!(f, "+{:x}", disp),
            },
            &Displacement::Zero => write!(f, ""),
        }
    }
}

impl Displacement {
    pub fn is_zero(&self) -> bool {
        match self {
            &Displacement::Word(disp) => disp == 0i16,
            &Displacement::Byte(disp) => disp == 0i8,
            _ => true,
        }
    }

    pub fn extend(&self) -> Self {
        match self {
            &Displacement::Word(disp) => Displacement::Word(i16::from(disp)),
            &Displacement::Byte(disp) => Displacement::Word(i16::from(disp)),
            _ => Self::Zero,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            &Displacement::Word(_) => 2,
            &Displacement::Byte(_) => 1,
            _ => 0,
        }
    }
}
