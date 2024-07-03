use crate::mmvm::numerical::Displacement;
use crate::mmvm::register::BaseRegister::{BP, BX};
use crate::mmvm::register::IndexRegister::{DI, SI};
use crate::mmvm::register::{BaseRegister, IndexRegister, Register};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Addressing {
    RegisterAddressing(Register),

    DirectAddressing(u16),

    DirectIndexAddressing(u16, u16),

    BasedAddressing(BaseRegister, bool, Displacement),

    IndexedAddressing(IndexRegister, bool, Displacement),

    BasedIndexedAddressing(BaseRegister, IndexRegister, bool, Displacement),
}

impl Addressing {
    fn decode_displacement(
        r#mod: u8,
        r_m: u8,
        binary_data: &[u8],
    ) -> (usize, bool, Option<Displacement>) {
        if binary_data.is_empty() {
            return (0, true, None);
        }
        match r#mod {
            0b00 => {
                if r_m != 0b110 {
                    (0, true, None)
                } else {
                    (
                        2,
                        true,
                        Some(Displacement::Word(u16::from(
                            ((binary_data[1] as u16) << 8) + (binary_data[0] as u16),
                        ))),
                    )
                }
            }
            0b01 => {
                if (binary_data[0] & 0b10000000) == 0b10000000 {
                    (
                        1,
                        false,
                        Some(Displacement::Byte(((binary_data[0] as i8) * -1i8) as u8)),
                    )
                } else {
                    (1, true, Some(Displacement::Byte(binary_data[0])))
                }
            }
            0b10 => {
                let mut displacement =
                    u16::from(((binary_data[1] as u16) << 8) + (binary_data[0] as u16));
                if (binary_data[1] & 0b10000000) != 0b0 {
                    displacement = ((displacement as i16) * -1i16) as u16;
                }
                (
                    2,
                    (binary_data[1] & 0b10000000) == 0b0,
                    Some(Displacement::Word(displacement)),
                )
            }
            _ => (0, true, None),
        }
    }

    pub fn decode_rm(w: u8, r#mod: u8, r_m: u8, binary_data: &[u8]) -> (usize, Option<Addressing>) {
        if r#mod == 0b11 {
            if let Some(register) = Register::decode(w == 0b1, true, r_m) {
                (0, Some(Addressing::RegisterAddressing(register)))
            } else {
                (0, None)
            }
        } else {
            if let (mut l, sign, Some(displacement)) =
                Self::decode_displacement(r#mod, r_m, binary_data)
            {
                if (displacement == Displacement::Byte(0b0))
                    | (displacement == Displacement::Word(0x0))
                {
                    l = 0;
                }
                let addressing = match r_m {
                    0b000 => Some(Addressing::BasedIndexedAddressing(
                        BX,
                        SI,
                        sign,
                        displacement,
                    )),
                    0b001 => Some(Addressing::BasedIndexedAddressing(
                        BX,
                        DI,
                        sign,
                        displacement,
                    )),
                    0b010 => Some(Addressing::BasedIndexedAddressing(
                        BP,
                        SI,
                        sign,
                        displacement,
                    )),
                    0b011 => Some(Addressing::BasedIndexedAddressing(
                        BP,
                        DI,
                        sign,
                        displacement,
                    )),
                    0b100 => Some(Addressing::IndexedAddressing(SI, sign, displacement)),
                    0b101 => Some(Addressing::IndexedAddressing(DI, sign, displacement)),
                    0b110 => {
                        if r#mod == 0b00 {
                            let address = match displacement {
                                Displacement::Word(displacement) => displacement,
                                _ => return (0, None),
                            };
                            Some(Addressing::DirectAddressing(address))
                        } else {
                            Some(Addressing::BasedAddressing(BP, sign, displacement))
                        }
                    }
                    0b111 => Some(Addressing::BasedAddressing(BX, sign, displacement)),
                    _ => None,
                };
                (l, addressing)
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

fn displacement_sign(sign: bool) -> &'static str {
    match sign {
        false => "-",
        true => "+",
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
            &Addressing::BasedAddressing(register, sign, displacement) => {
                if (displacement != Displacement::Byte(0b0))
                    & (displacement != Displacement::Word(0x0))
                {
                    write!(
                        f,
                        "[{}{}{}]",
                        register,
                        displacement_sign(sign),
                        displacement
                    )
                } else {
                    write!(f, "[{}]", register)
                }
            }
            &Addressing::IndexedAddressing(register, sign, displacement) => {
                if (displacement != Displacement::Byte(0b0))
                    & (displacement != Displacement::Word(0x0))
                {
                    write!(
                        f,
                        "[{}{}{}]",
                        register,
                        displacement_sign(sign),
                        displacement
                    )
                } else {
                    write!(f, "[{}]", register)
                }
            }
            &Addressing::BasedIndexedAddressing(base, index, sign, displacement) => {
                if (displacement != Displacement::Byte(0b0))
                    & (displacement != Displacement::Word(0x0))
                {
                    write!(
                        f,
                        "[{}+{}{}{}]",
                        base,
                        index,
                        displacement_sign(sign),
                        displacement
                    )
                } else {
                    write!(f, "[{}+{}]", base, index)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mmvm::{addressing::Addressing, numerical::Displacement};

    #[test]
    fn test_decode_displacement() {
        let testcases = [
            (0b00, 0b000, &[0x87, 0x54], 0, Displacement::Byte(0x0)),
            (0b00, 0b110, &[0x87, 0x54], 2, Displacement::Word(0x5487)),
            (0b01, 0b000, &[0x87, 0x00], 1, Displacement::Byte(0x79)),
            (0b10, 0b000, &[0x87, 0x54], 2, Displacement::Word(0x5487)),
            (0b11, 0b000, &[0x87, 0x54], 0, Displacement::Byte(0x0)),
        ];

        for (i, testcase) in testcases.into_iter().enumerate() {
            if let (l, _, Some(displacement)) =
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
            }
        }
    }

    // #[test]
    // fn test_decode() {
    //     let testcases = [(0b0, &[0x89, 0x54], 0b11111111, "al")];

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
