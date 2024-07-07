use address::Address;
use displacement::Displacement;
use ic::IC;
use immediate::Immediate;
use index::{Offset, Segment};
use port::Port;
use r#type::Type;
use std::fmt::{Display, Formatter};

pub mod address;
pub mod displacement;
pub mod ic;
pub mod immediate;
pub mod index;
pub mod port;
pub mod r#type;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Data<T>(pub T);

impl Display for Data<Immediate> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(immediate) = self;
        write!(f, "{}", immediate)
    }
}

impl Data<Immediate> {
    pub fn len(&self) -> usize {
        2
    }
}

impl Display for Data<Address> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(address) = self;
        write!(f, "{}", address)
    }
}

impl Data<Address> {
    pub fn len() -> usize {
        2
    }
}

impl Display for Data<Offset> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(offset) = self;
        write!(f, "{}", offset)
    }
}

impl Data<Offset> {
    pub fn len() -> usize {
        2
    }
}

impl Display for Data<Segment> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(segment) = self;
        write!(f, "{}", segment)
    }
}

impl Data<Segment> {
    pub fn len() -> usize {
        2
    }
}

impl Display for Data<Port> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(port) = self;
        write!(f, "{}", port)
    }
}

impl Data<Port> {
    pub fn len(&self) -> usize {
        1
    }
}

impl Display for Data<IC> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(ic) = self;
        write!(f, "{}", ic)
    }
}

impl Display for Data<Type> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(r#type) = self;
        write!(f, "{}", r#type)
    }
}

impl Data<Type> {
    pub fn len(&self) -> usize {
        1
    }
}

impl Display for Data<Displacement> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let &Data(displacement) = self;
        write!(f, "{}", displacement)
    }
}

impl Data<Displacement> {
    pub fn len(&self) -> usize {
        let &Data(displacement) = self;
        match displacement {
            Displacement::Word(_) => 2,
            Displacement::Byte(_) => 1,
            _ => 0,
        }
    }
}
