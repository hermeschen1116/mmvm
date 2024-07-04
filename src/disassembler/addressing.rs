use crate::disassembler::numerical::Displacement;
use crate::disassembler::register::BaseRegister::{BP, BX};
use crate::disassembler::register::IndexRegister::{DI, SI};
use crate::disassembler::register::{BaseRegister, IndexRegister, Register};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Addressing {
    RegisterAddressing(Register),

    DirectAddressing(u16),

    DirectIndexAddressing(u16, u16),

    BasedAddressing(BaseRegister, Displacement),

    IndexedAddressing(IndexRegister, Displacement),

    BasedIndexedAddressing(BaseRegister, IndexRegister, Displacement),
}

impl Addressing {
    fn decode_displacement(
        r#mod: u8,
        r_m: u8,
        binary_data: &[u8],
    ) -> (usize, Option<Displacement>) {
        if binary_data.is_empty() {
            return (0, None);
        }
        let (mut length, displacement) = match r#mod {
            0b00 => {
                if r_m != 0b110 {
                    return (0, Some(Displacement::Word(false, false, 0x0)));
                } else {
                    (2, Displacement::decode(&binary_data[..=1], false))
                }
            }
            0b01 => (1, Displacement::decode(&binary_data[0..1], true)),
            0b10 => (2, Displacement::decode(&binary_data[..=1], true)),
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
                            let address = match displacement {
                                Displacement::Word(_, _, disp) => disp,
                                _ => return (0, None),
                            };
                            (l, Some(Addressing::DirectAddressing(address)))
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
            &Addressing::DirectAddressing(address) => write!(f, "[{:04x}]", address),
            &Addressing::DirectIndexAddressing(offset, segment) => {
                write!(f, "{:x}:{:x}", offset, segment)
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

#[cfg(test)]
mod tests {
    use crate::disassembler::{addressing::Addressing, numerical::Displacement};

    #[test]
    fn test_decode_displacement() {
        let testcases = [
            (
                0b00,
                0b110,
                &[0x87, 0x54],
                2,
                Displacement::Word(false, true, 0x5487),
            ),
            (
                0b01,
                0b000,
                &[0x87, 0x00],
                1,
                Displacement::Byte(true, false, 0x79),
            ),
            (
                0b10,
                0b000,
                &[0x87, 0x54],
                2,
                Displacement::Word(true, true, 0x5487),
            ),
        ];

        for (i, testcase) in testcases.into_iter().enumerate() {
            if let (l, Some(displacement)) =
                Addressing::decode_displacement(testcase.0, testcase.1, testcase.2)
            {
                assert_eq!(
                    displacement, testcase.4,
                    "#{}, result: {}, expected: {}",
                    i, displacement, testcase.4
                );
                assert_eq!(
                    l, testcase.3,
                    "#{}, result: {}, expected: {}",
                    i, l, testcase.3
                );
            } else {
                panic!("#{}, result: None, expected: {}", i, testcase.4);
            }
        }
    }

    #[test]
    fn test_decode_rm() {
        let testcases = [
            (0b0, 0b11, 0b000, &[0x00, 0x00], 0, "al"),
            (0b1, 0b11, 0b001, &[0x00, 0x00], 0, "cx"),
            (0b0, 0b00, 0b000, &[0x87, 0x54], 0, "[bx+si]"),
            (0b0, 0b00, 0b110, &[0x87, 0x54], 2, "[5487]"),
            (0b0, 0b10, 0b000, &[0x00, 0x00], 0, "[bx+si]"),
            (0b0, 0b10, 0b000, &[0x87, 0x54], 2, "[bx+si+5487]"),
            (0b0, 0b10, 0b001, &[0x00, 0x00], 0, "[bx+di]"),
            (0b0, 0b10, 0b001, &[0x87, 0x54], 2, "[bx+di+5487]"),
            (0b0, 0b10, 0b010, &[0x00, 0x00], 0, "[bp+si]"),
            (0b0, 0b10, 0b010, &[0x87, 0x54], 2, "[bp+si+5487]"),
            (0b0, 0b10, 0b011, &[0x00, 0x00], 0, "[bp+di]"),
            (0b0, 0b10, 0b011, &[0x87, 0x54], 2, "[bp+di+5487]"),
            (0b0, 0b10, 0b100, &[0x00, 0x00], 0, "[si]"),
            (0b0, 0b10, 0b100, &[0x87, 0x54], 2, "[si+5487]"),
            (0b0, 0b10, 0b101, &[0x00, 0x00], 0, "[di]"),
            (0b0, 0b10, 0b101, &[0x87, 0x54], 2, "[di+5487]"),
            (0b0, 0b10, 0b110, &[0x00, 0x00], 0, "[bp]"),
            (0b0, 0b10, 0b110, &[0x87, 0x54], 2, "[bp+5487]"),
            (0b0, 0b10, 0b111, &[0x00, 0x00], 0, "[bx]"),
            (0b0, 0b10, 0b111, &[0x87, 0x54], 2, "[bx+5487]"),
            (0b0, 0b01, 0b000, &[0x87, 0x54], 1, "[bx+si-79]"),
            (0b0, 0b01, 0b001, &[0x87, 0x54], 1, "[bx+di-79]"),
            (0b0, 0b01, 0b010, &[0x87, 0x54], 1, "[bp+si-79]"),
            (0b0, 0b01, 0b011, &[0x87, 0x54], 1, "[bp+di-79]"),
            (0b0, 0b01, 0b100, &[0x87, 0x54], 1, "[si-79]"),
            (0b0, 0b01, 0b101, &[0x87, 0x54], 1, "[di-79]"),
            (0b0, 0b01, 0b110, &[0x87, 0x54], 1, "[bp-79]"),
            (0b0, 0b01, 0b111, &[0x87, 0x54], 1, "[bx-79]"),
            (0b0, 0b00, 0b111, &[0x87, 0x54], 0, "[bx]"),
        ];

        for (i, testcase) in testcases.into_iter().enumerate() {
            if let (l, Some(address)) =
                Addressing::decode_rm(testcase.0, testcase.1, testcase.2, testcase.3)
            {
                assert_eq!(
                    format!("{}", address),
                    testcase.5,
                    "#{}, result: {:?}, expected: {}",
                    i,
                    address,
                    testcase.5
                );
                assert_eq!(
                    l, testcase.4,
                    "#{}, {:?}, result: {:?}, expected: {}",
                    i, address, l, testcase.4
                );
            } else {
                panic!("#{}, result: None, expected: {}", i, testcase.5);
            }
        }
    }

    // #[test]
    // fn test_decode() {
    //     let testcases = [(0b0, &[0x87, 0x54], 0b11111111, "al")];

    //     for (i, testcase) in testcases.into_iter().enumerate() {
    //         match Addressing::decode(testcase.0, testcase.1, testcase.2) {
    //             (Some(reg), Some(r_m)) => {
    //                 assert_eq!(
    //                     format!("{}, {}", reg, r_m),
    //                     testcase.3,
    //                     "#{}, result: {}, {}, expected: {}",
    //                     i,
    //                     reg,
    //                     r_m,
    //                     testcase.3
    //                 )
    //             }
    //             (Some(reg), None) => {
    //                 assert_eq!(
    //                     format!("{}", reg),
    //                     testcase.3,
    //                     "#{}, result: {}, expected: {}",
    //                     i,
    //                     reg,
    //                     testcase.3
    //                 )
    //             }
    //             (None, Some(r_m)) => {
    //                 assert_eq!(
    //                     format!("{}", r_m),
    //                     testcase.3,
    //                     "#{}, result: {}, expected: {}",
    //                     i,
    //                     r_m,
    //                     testcase.3
    //                 )
    //             }
    //             _ => panic!("#{}, wrong input", i),
    //         }
    //     }
    // }
}
