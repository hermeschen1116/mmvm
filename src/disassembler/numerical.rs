use std::fmt::{Display, Formatter};

fn sign_check(sign: bool) -> &'static str {
    match sign {
        true => "+",
        false => "-",
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Displacement {
    Word(bool, bool, u16),
    Byte(bool, bool, u8),
}

impl Displacement {
    pub fn decode(value: &[u8], sign_enable: bool) -> Displacement {
        match value.len() {
            1 => match (sign_enable, (value[0] & 0x80) == 0b0) {
                (true, false) => Displacement::Byte(
                    sign_enable,
                    (value[0] & 0x80) == 0b0,
                    (!value[0] + 0b1) as u8,
                ),
                _ => Displacement::Byte(sign_enable, (value[0] & 0x80) == 0b0, value[0]),
            },
            2 => {
                let displacement = u16::from(((value[1] as u16) << 8) + (value[0] as u16));
                match (sign_enable, (value[1] & 0x80) == 0b0) {
                    (true, false) => Displacement::Word(
                        sign_enable,
                        (value[1] & 0x80) == 0b0,
                        ((displacement as i16) * -1i16) as u16,
                    ),
                    _ => Displacement::Word(sign_enable, (value[1] & 0x80) == 0b0, displacement),
                }
            }
            _ => panic!("input length should be 1 or 2"),
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            &Displacement::Word(_, _, disp) => disp == 0x0,
            &Displacement::Byte(_, _, disp) => disp == 0b0,
        }
    }

    pub fn add(&self, value: u16) -> Displacement {
        match self {
            &Displacement::Word(sign_enable, _, disp) => {
                let result = ((disp as i16) + (value as i16)) as u16;
                Displacement::Word(sign_enable, (result & 0x8000) == 0x0, result)
            }
            &Displacement::Byte(sign_enable, _, disp) => {
                let result = ((disp as i8) + (value as i8)) as u8;
                Displacement::Byte(sign_enable, (result & 0x80) == 0b0, result)
            }
        }
    }

    pub fn sub(&self, value: u16) -> Displacement {
        match self {
            &Displacement::Word(sign_enable, _, disp) => {
                let result = ((disp as i16) - (value as i16)) as u16;
                Displacement::Word(sign_enable, (result & 0x8000) == 0x0, result)
            }
            &Displacement::Byte(sign_enable, _, disp) => {
                let result = ((disp as i8) - (value as i8)) as u8;
                Displacement::Byte(sign_enable, (result & 0x80) == 0b0, result)
            }
        }
    }
}

impl Display for Displacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Displacement::Word(sign_enable, sign, displacement) => {
                if self.is_zero() | !sign_enable {
                    write!(f, "{:x}", displacement)
                } else {
                    write!(f, "{}{:x}", sign_check(sign), displacement)
                }
            }
            &Displacement::Byte(sign_enable, sign, displacement) => {
                if self.is_zero() | !sign_enable {
                    write!(f, "{:x}", displacement)
                } else {
                    write!(f, "{}{:x}", sign_check(sign), displacement)
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Immediate {
    Word(bool, bool, u16),
    Byte(bool, bool, u8),
}

impl Immediate {
    pub fn decode(value: &[u8], sign_enable: bool) -> Immediate {
        match value.len() {
            1 => match (sign_enable, (value[0] & 0x80) == 0b0) {
                (true, false) => Immediate::Byte(
                    sign_enable,
                    (value[0] & 0x80) == 0b0,
                    (!value[0] + 0b1) as u8,
                ),
                _ => Immediate::Byte(sign_enable, (value[0] & 0x80) == 0b0, value[0]),
            },
            2 => {
                let displacement = u16::from(((value[1] as u16) << 8) + (value[0] as u16));
                match (sign_enable, (value[1] & 0x80) == 0b0) {
                    (true, false) => Immediate::Word(
                        sign_enable,
                        (value[1] & 0x80) == 0b0,
                        ((displacement as i16) * -1i16) as u16,
                    ),
                    _ => Immediate::Word(sign_enable, (value[1] & 0x80) == 0b0, displacement),
                }
            }
            _ => panic!("input length should be 1 or 2"),
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            &Immediate::Word(_, _, imme) => imme == 0x0,
            &Immediate::Byte(_, _, imme) => imme == 0b0,
        }
    }

    pub fn add(&self, value: u16) -> Immediate {
        match self {
            &Immediate::Word(sign_enable, _, imme) => {
                let result = ((imme as i16) + (value as i16)) as u16;
                Immediate::Word(sign_enable, (result & 0x8000) == 0x0, result)
            }
            &Immediate::Byte(sign_enable, _, imme) => {
                let result = ((imme as i8) + (value as i8)) as u8;
                Immediate::Byte(sign_enable, (result & 0x80) == 0b0, result)
            }
        }
    }

    pub fn sub(&self, value: u16) -> Immediate {
        match self {
            &Immediate::Word(sign_enable, _, imme) => {
                let result = ((imme as i16) - (value as i16)) as u16;
                Immediate::Word(sign_enable, (result & 0x8000) == 0x0, result)
            }
            &Immediate::Byte(sign_enable, _, imme) => {
                let result = ((imme as i8) - (value as i8)) as u8;
                Immediate::Byte(sign_enable, (result & 0x80) == 0b0, result)
            }
        }
    }

    pub fn extend(&self) -> Immediate {
        match self {
            &Immediate::Word(_, _, _) => *self,
            &Immediate::Byte(sign_enable, sign, imme) => {
                if !sign {
                    Immediate::Word(sign_enable, sign, (imme as u16) | 0xff00)
                } else {
                    Immediate::Word(sign_enable, sign, imme as u16)
                }
            }
        }
    }
}

impl Display for Immediate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Immediate::Word(sign_enable, sign, immediate) => match sign_enable & !sign {
                true => write!(f, "{}{:04x}", sign_check(sign), immediate),
                false => write!(f, "{:04x}", immediate),
            },
            &Immediate::Byte(sign_enable, sign, immediate) => match sign_enable & !sign {
                true => write!(f, "{}{:x}", sign_check(sign), immediate),
                false => write!(f, "{:x}", immediate),
            },
        }
    }
}
