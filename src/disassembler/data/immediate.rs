use std::cmp;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    UnsignedWord(u16),
    SignedWord(i16),
    UnsignedByte(u8),
    SignedByte(i8),
}

impl Immediate {
    pub fn new(value: &[u8], sign: bool) -> Self {
        match value.len() {
            1 => {
                let immediate = u8::from_le_bytes(value.try_into().unwrap());
                match sign {
                    true => Immediate::SignedByte(immediate as i8),
                    false => Immediate::UnsignedByte(immediate),
                }
            }
            2 => {
                let immediate = u16::from_le_bytes(value.try_into().unwrap());
                match sign {
                    true => Immediate::SignedWord(immediate as i16),
                    false => Immediate::UnsignedWord(immediate),
                }
            }
            _ => panic!("length of value should be 1 or 2"),
        }
    }

    pub fn extend(&self) -> Self {
        match self {
            &Immediate::UnsignedByte(immediate) => Immediate::UnsignedWord(u16::from(immediate)),
            &Immediate::SignedByte(immediate) => Immediate::SignedWord(i16::from(immediate)),
            &immediate => immediate,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            &Immediate::UnsignedWord(immediate) => immediate == 0b0,
            &Immediate::SignedWord(immediate) => immediate == 0b0,
            &Immediate::UnsignedByte(immediate) => immediate == 0x0,
            &Immediate::SignedByte(immediate) => immediate == 0x0,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            &Immediate::UnsignedWord(_) | &Immediate::SignedWord(_) => 2,
            &Immediate::UnsignedByte(_) | &Immediate::SignedByte(_) => 1,
        }
    }
}

impl Display for Immediate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Immediate::UnsignedWord(immediate) => write!(f, "{:04x}", immediate),
            &Immediate::SignedWord(immediate) => match immediate.cmp(&0i16) {
                cmp::Ordering::Less => write!(f, "-{:04x}", !(immediate - 0b1)),
                cmp::Ordering::Equal => write!(f, "{:04x}", immediate),
                cmp::Ordering::Greater => write!(f, "+{:04x}", immediate),
            },
            &Immediate::UnsignedByte(immediate) => write!(f, "{:x}", immediate),
            &Immediate::SignedByte(immediate) => match immediate.cmp(&0i8) {
                cmp::Ordering::Less => write!(f, "-{:x}", !(immediate - 0b1)),
                cmp::Ordering::Equal => write!(f, "{:x}", immediate),
                cmp::Ordering::Greater => write!(f, "+{:x}", immediate),
            },
        }
    }
}
