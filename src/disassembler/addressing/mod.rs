use crate::disassembler::addressing::{addressing::Addressing, register::Register};

pub mod addressing;
pub mod register;

pub fn decode(w: u8, binary_data: &[u8], mask: u8) -> (Option<Addressing>, Option<Addressing>) {
    if binary_data.is_empty() {
        panic!("there's no enough length to decode data")
    }
    match mask {
        0b11111111 => {
            let r#mod = (binary_data[0] & 0b11000000) >> 6;
            let reg = (binary_data[0] & 0b00111000) >> 3;
            let r_m = binary_data[0] & 0b00000111;
            let address = Addressing::new(w, r#mod, r_m, &binary_data[1..]);
            if let Some(register) = Register::decode(w == 0b1, true, reg) {
                (
                    Some(Addressing::RegisterAddressing(register)),
                    Some(address),
                )
            } else {
                (None, None)
            }
        }
        0b11011111 => {
            let r#mod = (binary_data[0] & 0b11000000) >> 6;
            let reg = (binary_data[0] & 0b00111000) >> 3;
            let r_m = binary_data[0] & 0b00000111;
            let address = Addressing::new(w, r#mod, r_m, &binary_data[1..]);
            if let Some(register) = Register::decode(true, false, reg) {
                (
                    Some(Addressing::RegisterAddressing(register)),
                    Some(address),
                )
            } else {
                (None, None)
            }
        }
        0b11000111 => {
            let r#mod = (binary_data[0] & 0b11000000) >> 6;
            let r_m = binary_data[0] & 0b00000111;
            let address = Addressing::new(w, r#mod, r_m, &binary_data[1..]);
            (None, Some(address))
        }
        0b00000111 => {
            let reg = binary_data[0] & mask;
            if let Some(register) = Register::decode(true, true, reg) {
                (Some(Addressing::RegisterAddressing(register)), None)
            } else {
                (None, None)
            }
        }
        0b00011000 => {
            let reg = (binary_data[0] & mask) >> 3;
            if let Some(register) = Register::decode(w == 0b1, false, reg) {
                (Some(Addressing::RegisterAddressing(register)), None)
            } else {
                (None, None)
            }
        }
        _ => panic!("mask should be 0b11111111, 0b11011111, 0b11000111, 0b00000111, or 0b00011000"),
    }
}
