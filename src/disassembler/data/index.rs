use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Offset(u16);

impl Offset {
    pub fn new(offset_low: u8, offset_high: u8) -> Self {
        Self(u16::from_le_bytes([offset_low, offset_high]))
    }

    pub fn len() -> usize {
        2
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Offset(offset) = self;
        write!(f, "{:04x}", offset)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Segment(u16);

impl Segment {
    pub fn new(seg_low: u8, seg_high: u8) -> Self {
        Self(u16::from_le_bytes([seg_low, seg_high]))
    }

    pub fn len() -> usize {
        2
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Segment(segment) = self;
        write!(f, "{:04x}", segment)
    }
}
