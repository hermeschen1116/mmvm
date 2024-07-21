use crate::disassembler::addressing::Addressing;
use crate::disassembler::direction::Direction;

use crate::disassembler::numerical::{Immediate, Numerical};
use crate::disassembler::register::SegmentRegister::DS;
use crate::disassembler::register::WordRegister::{BP, BX, DI, SI};
use crate::disassembler::register::{BaseRegister, IndexRegister, Register};
use crate::interpreter::hardware::Hardware;

pub fn calculate_effective_address(r_m: &Addressing, hardware: &Hardware) -> Option<u16> {
    match r_m {
        &Addressing::RegisterAddressing(_) => None,
        &Addressing::DirectAddressing(Numerical::Imme(Immediate::UnsignedWord(addr))) => Some(addr),
        &Addressing::DirectIndexAddressing(
            Numerical::Imme(Immediate::UnsignedWord(offset)),
            Numerical::Imme(Immediate::UnsignedWord(segment)),
        ) => Some(segment << 4 + offset),
        &Addressing::BasedAddressing(base, Numerical::Disp(disp)) => {
            let address = ((hardware.clone().read_from_segment_register(DS) << 4) as i16
                + match base {
                    BaseRegister::BX => hardware.clone().read_from_word_register(BX) as i16,
                    BaseRegister::BP => hardware.clone().read_from_word_register(BP) as i16,
                }
                + match disp {
                    crate::disassembler::numerical::Displacement::SignedWord(disp) => disp,
                    crate::disassembler::numerical::Displacement::SignedByte(disp) => {
                        i16::from(disp)
                    }
                    _ => return None,
                }) as u16;
            Some(address)
        }
        &Addressing::IndexedAddressing(index, Numerical::Disp(disp)) => {
            let address = ((hardware.clone().read_from_segment_register(DS) << 4) as i16
                + match index {
                    IndexRegister::DI => hardware.clone().read_from_word_register(DI) as i16,
                    IndexRegister::SI => hardware.clone().read_from_word_register(SI) as i16,
                }
                + match disp {
                    crate::disassembler::numerical::Displacement::SignedWord(disp) => disp,
                    crate::disassembler::numerical::Displacement::SignedByte(disp) => {
                        i16::from(disp)
                    }
                    _ => return None,
                }) as u16;
            Some(address)
        }
        &Addressing::BasedIndexedAddressing(base, index, Numerical::Disp(disp)) => {
            let address = ((hardware.clone().read_from_segment_register(DS) << 4) as i16
                + match base {
                    BaseRegister::BX => hardware.clone().read_from_word_register(BX) as i16,
                    BaseRegister::BP => hardware.clone().read_from_word_register(BP) as i16,
                }
                + match index {
                    IndexRegister::DI => hardware.clone().read_from_word_register(DI) as i16,
                    IndexRegister::SI => hardware.clone().read_from_word_register(SI) as i16,
                }
                + match disp {
                    crate::disassembler::numerical::Displacement::SignedWord(disp) => disp,
                    crate::disassembler::numerical::Displacement::SignedByte(disp) => {
                        i16::from(disp)
                    }
                    _ => return None,
                }) as u16;
            Some(address)
        }
        _ => unreachable!(),
    }
}

pub fn read_from_address(
    word_mode: bool,
    address: &Addressing,
    hardware: &Hardware,
) -> Option<Immediate> {
    match address {
        &Addressing::RegisterAddressing(reg) => match reg {
            Register::ByteReg(reg) => Some(Immediate::UnsignedByte(
                hardware.clone().read_from_byte_register(reg),
            )),
            Register::WordReg(reg) => Some(Immediate::UnsignedWord(
                hardware.clone().read_from_word_register(reg),
            )),
            Register::SegmentReg(reg) => Some(Immediate::UnsignedWord(
                hardware.clone().read_from_segment_register(reg),
            )),
        },
        _ => {
            if let Some(address) = calculate_effective_address(&address, hardware) {
                match word_mode {
                    true => Some(Immediate::UnsignedWord(
                        hardware
                            .clone()
                            .read_word_from_memory(address)
                            .expect("Unable to retrieve data"),
                    )),
                    false => Some(Immediate::UnsignedByte(
                        hardware
                            .clone()
                            .read_byte_from_memory(address)
                            .expect("Unable to retrieve data"),
                    )),
                }
            } else {
                None
            }
        }
    }
}

pub fn write_to_address(address: &Addressing, immediate: &Immediate, hardware: &mut Hardware) {
    match address {
        &Addressing::RegisterAddressing(reg) => match (reg, immediate) {
            (Register::ByteReg(reg), &Immediate::SignedByte(imme)) => {
                hardware.write_to_byte_register(reg, imme as u8);
            }
            (Register::ByteReg(reg), &Immediate::UnsignedByte(imme)) => {
                hardware.write_to_byte_register(reg, imme)
            }
            (Register::WordReg(reg), &Immediate::SignedWord(imme)) => {
                hardware.write_to_word_register(reg, imme as u16)
            }
            (Register::WordReg(reg), &Immediate::UnsignedWord(imme)) => {
                hardware.write_to_word_register(reg, imme)
            }
            (Register::SegmentReg(reg), &Immediate::UnsignedWord(imme)) => {
                hardware.write_to_segment_register(reg, imme)
            }
            _ => panic!("Register size and immediate size should be the same"),
        },
        _ => {
            if let Some(addr) = calculate_effective_address(address, hardware) {
                match immediate {
                    &Immediate::UnsignedWord(imme) => hardware.write_word_to_memory(addr, imme),
                    &Immediate::SignedWord(imme) => {
                        hardware.write_word_to_memory(addr, imme as u16)
                    }
                    &Immediate::UnsignedByte(imme) => hardware.write_byte_to_memory(addr, imme),
                    &Immediate::SignedByte(imme) => hardware.write_byte_to_memory(addr, imme as u8),
                }
            } else {
                panic!("No match address type")
            }
        }
    }
}

pub fn write_from_address_to_address(
    direction: Direction,
    reg: &Addressing,
    r_m: &Addressing,
    hardware: &mut Hardware,
) {
    let word_mode = match reg {
        &Addressing::RegisterAddressing(Register::ByteReg(_)) => false,
        &Addressing::RegisterAddressing(Register::WordReg(_))
        | &Addressing::RegisterAddressing(Register::SegmentReg(_)) => true,
        _ => unreachable!(),
    };
    match direction {
        Direction::FromReg => {
            if let Some(imme) = read_from_address(word_mode, reg, hardware) {
                write_to_address(&r_m, &imme, hardware)
            } else {
                panic!("Nothing in {:?}", reg)
            }
        }
        Direction::ToReg => {
            if let Some(imme) = read_from_address(word_mode, r_m, hardware) {
                write_to_address(&reg, &imme, hardware)
            } else {
                panic!("Nothing in {:?}", r_m)
            }
        }
    }
}
