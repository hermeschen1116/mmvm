use crate::disassembler::addressing::register::BaseRegister::{BP, BX};
use crate::disassembler::addressing::register::IndexRegister::{DI, SI};
use crate::disassembler::addressing::register::{BaseRegister, IndexRegister, Register};
use crate::disassembler::data::address::Address;
use crate::disassembler::data::displacement::Displacement;
use crate::disassembler::data::index::{Offset, Segment};
use crate::disassembler::data::Data;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Addressing {
    RegisterAddressing(Register),

    DirectAddressing(Data<Address>),

    DirectIndexAddressing(Data<Offset>, Data<Segment>),

    BasedAddressing(BaseRegister, Data<Displacement>),

    IndexedAddressing(IndexRegister, Data<Displacement>),

    BasedIndexedAddressing(BaseRegister, IndexRegister, Data<Displacement>),
}

impl Addressing {
    pub fn new(w: u8, r#mod: u8, r_m: u8, data: &[u8]) -> Self {
        if (r#mod <= 0b00) & (r#mod >= 0b11) {
            panic!("mod should be 0b00, 0b01, 0b10, or 0b11")
        }
        if data.is_empty() {
            panic!("there's no enough length to decode data")
        }

        if r#mod == 0b11 {
            return Addressing::RegisterAddressing(Register::decode(w == 0b1, true, r_m).unwrap());
        }
        if (r#mod == 0b00) & (r_m == 0b110) {
            let address = Address::new(data[0], data[1]);
            return Addressing::DirectAddressing(Data(address));
        }

        let displacement = if r#mod != 0b00 {
            let disp = match w {
                0b0 => Displacement::from(data[0]),
                0b1 => Displacement::from(&data[0..2]),
                _ => panic!("w should be 0b0 or 0b1"),
            };

            if disp.is_zero() {
                Data(Displacement::Zero)
            } else {
                Data(disp)
            }
        } else {
            Data(Displacement::Zero)
        };
        match r_m {
            0b000 => Addressing::BasedIndexedAddressing(BX, SI, displacement),
            0b001 => Addressing::BasedIndexedAddressing(BX, DI, displacement),
            0b010 => Addressing::BasedIndexedAddressing(BP, SI, displacement),
            0b011 => Addressing::BasedIndexedAddressing(BP, DI, displacement),
            0b100 => Addressing::IndexedAddressing(SI, displacement),
            0b101 => Addressing::IndexedAddressing(DI, displacement),
            0b110 => Addressing::BasedAddressing(BP, displacement),
            0b111 => Addressing::BasedAddressing(BX, displacement),
            _ => panic!("r_m should be 0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b110, or 0b111"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            &Addressing::RegisterAddressing(_) => 0,
            &Addressing::DirectAddressing(_) => 2,
            &Addressing::DirectIndexAddressing(_, _) => 4,
            &Addressing::BasedAddressing(_, Data(disp)) => disp.len(),
            &Addressing::IndexedAddressing(_, Data(disp)) => disp.len(),
            &Addressing::BasedIndexedAddressing(_, _, Data(disp)) => disp.len(),
        }
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Addressing::RegisterAddressing(register) => write!(f, "{}", register),
            &Addressing::DirectAddressing(address) => write!(f, "[{}]", address),
            &Addressing::DirectIndexAddressing(offset, segment) => {
                write!(f, "{}:{}", offset, segment)
            }
            &Addressing::BasedAddressing(register, displacement) => {
                write!(f, "[{}{}]", register, displacement)
            }
            &Addressing::IndexedAddressing(register, displacement) => {
                write!(f, "[{}{}]", register, displacement)
            }
            &Addressing::BasedIndexedAddressing(base, index, displacement) => {
                write!(f, "[{}+{}{}]", base, index, displacement)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::disassembler::addressing::addressing::Addressing;

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
            let r_m = Addressing::new(testcase.0, testcase.1, testcase.2, testcase.3);
            assert_eq!(
                format!("{}", r_m),
                testcase.5,
                "#{}, result: {:?}, expected: {}",
                i,
                r_m,
                testcase.5
            );
            assert_eq!(
                r_m.len(),
                testcase.4,
                "#{}, {:?}, result: {:?}, expected: {}",
                i,
                r_m,
                r_m.len(),
                testcase.4
            );
        }
    }
}
