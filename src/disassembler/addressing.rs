use crate::disassembler::numerical::{Displacement, Numerical};
use crate::disassembler::register::BaseRegister::{BP, BX};
use crate::disassembler::register::IndexRegister::{DI, SI};
use crate::disassembler::register::{BaseRegister, IndexRegister, Register};
use std::fmt::{Display, Formatter};

use super::numerical::Immediate;

#[derive(Debug, Copy, Clone)]
pub enum Addressing {
    RegisterAddressing(Register),

    DirectAddressing(Numerical),

    DirectIndexAddressing(Numerical, Numerical),

    BasedAddressing(BaseRegister, Numerical),

    IndexedAddressing(IndexRegister, Numerical),

    BasedIndexedAddressing(BaseRegister, IndexRegister, Numerical),
}

impl Addressing {
    pub fn decode_displacement(
        r#mod: u8,
        r_m: u8,
        binary_data: &[u8],
    ) -> (usize, Option<Numerical>) {
        if binary_data.is_empty() {
            return (0, None);
        }
        let (mut length, displacement) = match r#mod {
            0b00 => {
                if r_m != 0b110 {
                    return (
                        0,
                        Some(Numerical::Imme(Immediate::from(&[0x00, 0x00], true))),
                    );
                } else {
                    (
                        2,
                        Numerical::Imme(Immediate::from(&binary_data[0..2], false)),
                    )
                }
            }
            0b01 => (
                1,
                Numerical::Disp(Displacement::from(&binary_data[0..1], true)),
            ),
            0b10 => (
                2,
                Numerical::Disp(Displacement::from(&binary_data[0..2], true)),
            ),
            _ => return (0, None),
        };
        if (length != 0) & displacement.is_zero() {
            length = 0
        }
        (length, Some(displacement))
    }

    pub fn decode_rm(w: u8, r#mod: u8, r_m: u8, binary_data: &[u8]) -> (usize, Option<Addressing>) {
        if binary_data.is_empty() {
            return (0, None);
        }
        if r#mod == 0b11 {
            if let Some(register) = Register::decode(w == 0b1, true, r_m) {
                (0, Some(Addressing::RegisterAddressing(register)))
            } else {
                (0, None)
            }
        } else {
            if let (l, Some(displacement)) = Self::decode_displacement(r#mod, r_m, binary_data) {
                match r_m {
                    0b000 => (
                        l,
                        Some(Addressing::BasedIndexedAddressing(BX, SI, displacement)),
                    ),
                    0b001 => (
                        l,
                        Some(Addressing::BasedIndexedAddressing(BX, DI, displacement)),
                    ),
                    0b010 => (
                        l,
                        Some(Addressing::BasedIndexedAddressing(BP, SI, displacement)),
                    ),
                    0b011 => (
                        l,
                        Some(Addressing::BasedIndexedAddressing(BP, DI, displacement)),
                    ),
                    0b100 => (l, Some(Addressing::IndexedAddressing(SI, displacement))),
                    0b101 => (l, Some(Addressing::IndexedAddressing(DI, displacement))),
                    0b110 => {
                        if r#mod == 0b00 {
                            (l, Some(Addressing::DirectAddressing(displacement)))
                        } else {
                            (l, Some(Addressing::BasedAddressing(BP, displacement)))
                        }
                    }
                    0b111 => (l, Some(Addressing::BasedAddressing(BX, displacement))),
                    _ => (0, None),
                }
            } else {
                (0, None)
            }
        }
    }

    pub fn decode(
        w: u8,
        binary_data: &[u8],
        mask: u8,
    ) -> (usize, Option<Addressing>, Option<Addressing>) {
        if binary_data.is_empty() {
            return (0, None, None);
        }
        match mask {
            0b11111111 => {
                let r#mod = (binary_data[0] & 0b11000000) >> 6;
                let reg = (binary_data[0] & 0b00111000) >> 3;
                let r_m = binary_data[0] & 0b00000111;
                if let (l, Some(address)) = Self::decode_rm(w, r#mod, r_m, &binary_data[1..]) {
                    if let Some(register) = Register::decode(w == 0b1, true, reg) {
                        (
                            l,
                            Some(Addressing::RegisterAddressing(register)),
                            Some(address),
                        )
                    } else {
                        (0, None, None)
                    }
                } else {
                    (0, None, None)
                }
            }
            0b11011111 => {
                let r#mod = (binary_data[0] & 0b11000000) >> 6;
                let reg = (binary_data[0] & 0b00111000) >> 3;
                let r_m = binary_data[0] & 0b00000111;
                if let (l, Some(address)) = Self::decode_rm(w, r#mod, r_m, &binary_data[1..]) {
                    if let Some(register) = Register::decode(true, false, reg) {
                        (
                            l,
                            Some(Addressing::RegisterAddressing(register)),
                            Some(address),
                        )
                    } else {
                        (0, None, None)
                    }
                } else {
                    (0, None, None)
                }
            }
            0b11000111 => {
                let r#mod = (binary_data[0] & 0b11000000) >> 6;
                let r_m = binary_data[0] & 0b00000111;
                if let (l, Some(address)) = Self::decode_rm(w, r#mod, r_m, &binary_data[1..]) {
                    (l, None, Some(address))
                } else {
                    (0, None, None)
                }
            }
            0b00000111 => {
                let reg = binary_data[0] & mask;
                if let Some(register) = Register::decode(true, true, reg) {
                    (0, Some(Addressing::RegisterAddressing(register)), None)
                } else {
                    (0, None, None)
                }
            }
            0b00011000 => {
                let reg = (binary_data[0] & mask) >> 3;
                if let Some(register) = Register::decode(w == 0b1, false, reg) {
                    (0, Some(Addressing::RegisterAddressing(register)), None)
                } else {
                    (0, None, None)
                }
            }
            _ => (0, None, None),
        }
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Addressing::RegisterAddressing(register) => write!(f, "{}", register),
            &Addressing::DirectAddressing(address) => write!(f, "[{}]", address),
            &Addressing::DirectIndexAddressing(offset, segment) => {
                write!(f, "{}:{}", segment, offset)
            }
            &Addressing::BasedAddressing(register, displacement) => {
                if !displacement.is_zero() {
                    write!(f, "[{}{}]", register, displacement)
                } else {
                    write!(f, "[{}]", register)
                }
            }
            &Addressing::IndexedAddressing(register, displacement) => {
                if !displacement.is_zero() {
                    write!(f, "[{}{}]", register, displacement)
                } else {
                    write!(f, "[{}]", register)
                }
            }
            &Addressing::BasedIndexedAddressing(base, index, displacement) => {
                if !displacement.is_zero() {
                    write!(f, "[{}+{}{}]", base, index, displacement)
                } else {
                    write!(f, "[{}+{}]", base, index)
                }
            }
        }
    }
}
