use core::error::Source;
use std::intrinsics::unreachable;
use std::ops::{BitAnd, BitOr, BitXor};
use std::{clone, result};

use crate::disassembler::direction::Direction;
use crate::disassembler::numerical::Immediate;
use crate::disassembler::register::ByteRegister::{AH, AL, CL};
use crate::disassembler::register::Register;
use crate::disassembler::register::WordRegister::{AX, DX};

use crate::interpreter::{hardware::Hardware, utils::*};
use crate::utils::header::Header;

fn match_reg(binary_data: u8, reference: &[u8]) -> bool {
    let reg = (binary_data & 0b00111000) >> 3;
    reference.contains(&reg)
}

pub fn decode_data(
    word_mode: bool,
    sign_enable: bool,
    binary_data: &[u8],
) -> (usize, Option<Numerical>) {
    if binary_data.is_empty() {
        return (0, None);
    }
    if !word_mode {
        (
            1,
            Some(Numerical::Imme(Immediate::from(
                &binary_data[0..1],
                sign_enable,
            ))),
        )
    } else {
        (
            2,
            Some(Numerical::Imme(Immediate::from(
                &binary_data[0..2],
                sign_enable,
            ))),
        )
    }
}

pub fn execute_move_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // Register/Memory to/from Register
        0b10001000..=0b10001011 => {
            let d = (binary_data[0] & 0b00000010) >> 1;
            let w = binary_data[0] & 0b00000001;
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11111111)
            {
                write_from_address_to_address(Direction::from(d), reg, r_m, hardware)
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate to Register/Memory
        0b11000110 | 0b11000111 if match_reg(binary_data[1], &[0b000]) => {
            let w = binary_data[0] & 0b00000001;
            if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                if let (_, Some(immediate)) = decode_data(w == 0b1, false, &binary_data[(2 + rl)..])
                {
                    write_to_address(r_m, immediate, hardware)
                } else {
                    panic!("Instruction decode error")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate to Register
        0b10110000..=0b10111111 => {
            let w = (binary_data[0] & 0b00001000) >> 3;
            if let (_, Some(reg), None) = Addressing::decode(w, binary_data, 0b00000111) {
                if let (dl, Some(immediate)) = decode_data(w == 0b1, false, &binary_data[1..]) {
                    write_to_address(reg, immediate, hardware)
                } else {
                    panic!("Instruction decode error")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // Memory <-> Accumulator
        0b10100000 | 0b10100001 | 0b10100010 | 0b10100011 => {
            let d = (binary_data[0] & 0b00000010) >> 1;
            let w = binary_data[0] & 0b00000001;
            if let (_, Some(address)) = decode_data(true, false, &binary_data[1..]) {
                write_from_address_to_address(
                    Direction::from(d),
                    Addressing::RegisterAddressing(
                        Register::decode(w == 0b1, true, 0b000).unwrap(),
                    ),
                    Addressing::DirectAddressing(address),
                    hardware,
                )
            } else {
                panic!("Instruction decode error")
            }
        }
        // Register/Memory <-> Segment Register
        0b10001110 | 0b10001100 if (binary_data[1] & 0b00100000) == 0b00000000 => {
            let d = (binary_data[0] & 0b00000010) >> 1;
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(0, &binary_data[1..], 0b11011111)
            {
                write_from_address_to_address(Direction::from(d), reg, r_m, hardware)
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_push_pop_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // PUSH (Register/Memory)
        0b11111111 if (binary_data[1] & 0b00111000) == 0b00110000 => {
            if let (_, None, Some(r_m)) = Addressing::decode(0, &binary_data[1..], 0b11000111) {
                if let Some(Immediate::UnsignedWord(imme)) = read_from_address(true, &r_m, hardware)
                {
                    hardware.push_to_stack(imme)
                } else {
                    unreachable!()
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // PUSH (Register)
        0b01010000..=0b01010111 => {
            if let (_, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                if let Some(Immediate::UnsignedWord(imme)) = read_from_address(true, &reg, hardware)
                {
                    hardware.push_to_stack(imme)
                } else {
                    unreachable!()
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // POP (Register/Memory)
        0b10001111 if (binary_data[1] & 0b00111000) == 0b00000000 => {
            if let (_, None, Some(r_m)) = Addressing::decode(0, &binary_data[1..], 0b11000111) {
                if let Some(imme) = hardware.pop_from_stack() {
                    write_to_address(&r_m, &Immediate::UnsignedWord(imme), hardware)
                } else {
                    panic!("Not enough data")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // POP (Register)
        0b01011000..=0b01011111 => {
            if let (_, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                if let Some(imme) = hardware.pop_from_stack() {
                    write_to_address(&reg, &Immediate::UnsignedWord(imme), hardware)
                } else {
                    panic!("Not enough data")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // PUSH / POP (Segment Register)
        _ => {
            if let (_, Some(reg), None) = Addressing::decode(0, &binary_data[1..], 0b00011000) {
                if (binary_data[0] & 0b11100111) == 0b00000110 {
                    if let Some(Immediate::UnsignedWord(imme)) =
                        read_from_address(true, &reg, hardware)
                    {
                        hardware.push_to_stack(imme);
                    } else {
                        unreachable!()
                    }
                } else if (binary_data[0] & 0b11100111) == 0b00000111 {
                    if let Some(imme) = hardware.pop_from_stack() {
                        write_to_address(&reg, &Immediate::UnsignedWord(imme), hardware)
                    } else {
                        panic!("Not enough data")
                    }
                } else {
                    return panic!("Instruction decode error");
                }
            } else {
                panic!("Instruction decode error")
            }
        }
    }
}

pub fn execute_exchange_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // Register/Memory with Register
        0b10000110 | 0b10000111 => {
            let w = binary_data[0] & 0b00000001;
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11111111)
            {
                let imme = read_from_address(w == 0b1, r_m, hardware)
                    .unwrap_or(panic!("Unable to retrive data"));
                write_from_address_to_address(Direction::FromReg, &reg, &r_m, hardware);
                write_to_address(&reg, &imme, hardware);
            } else {
                panic!("Instruction decode error")
            }
        }
        // Register with Accumulator
        0b10010000..=0b10010111 => {
            if let (_, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                let imme = read_from_address(true, &reg, hardware)
                    .unwrap_or(panic!("Unable to retrive data"));
                let accumulator = Addressing::RegisterAddressing(Register::WordReg(AX));
                write_from_address_to_address(Direction::FromReg, &accumulator, &reg, hardware);
                write_to_address(&reg, &imme, hardware);
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_load_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    if let (_, Some(reg), Some(r_m)) = Addressing::decode(0b1, &binary_data[1..], 0b11111111) {
        match binary_data[0] {
            0b10001101 => write_from_address_to_address(Direction::ToReg, reg, r_m, hardware),
            0b11000101 => todo!("LDS"),
            0b11000100 => todo!("LES"),
            _ => return panic!("Instruction decode error"),
        };
    } else {
        panic!("Instruction decode error")
    }
}

pub fn execute_arithmic_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    match binary_data[0] {
        // Reg./Memory with Register to Either
        0b00000000..=0b00000011
        | 0b00010000..=0b00010011
        | 0b00101000..=0b00101011
        | 0b00011000..=0b00011011
        | 0b00111000..=0b00111011 => {
            let d = (binary_data[0] & 0b00000010) >> 1;
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11111111)
            {
                let value_1 = read_from_address(w == 0b1, &reg, hardware)
                    .expect("Unable to retrieve data");
                let value_2 = read_from_address(w == 0b1, &r_m, hardware)
                    .expect("Unable to retrieve data");
                match (value_1, value_2) {
                	(Immediate::UnsignedByte(value_1), Immediate::UnsignedByte(value_2)) => {
                 	let (result, overflow) = match (binary_data[0] & 0b00111000) >> 3 {
                     0b000 => {
                     	let (result, overflow) = i8::from(value_1).overflowing_add(i8::from(value_2));
                      	match Direction::from(d) {
                        Direction::FromReg => write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware),
                        Direction::ToReg => write_to_address(&reg, &Immediate::UnsignedByte(result as u8), hardware),
                    };
                    (result, overflow)
                     }
                     0b010 => {
                     	let (result, _) = i8::from(value_1).overflowing_add(i8::from(value_2));
                      	let (result, overflow) = result.overflowing_add(i8::from(hardware.clone().read_flags("CF")));
                      	match Direction::from(d) {
                        Direction::FromReg => write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware),
                        Direction::ToReg => write_to_address(&reg, &Immediate::UnsignedByte(result as u8), hardware),
                    };
                    (result, overflow)
                     }
                     0b101 => {
                     	let (result, _) = i8::from(value_1).overflowing_add(i8::from(value_2));
                      	let (result, overflow) = result.overflowing_add(i8::from(hardware.clone().read_flags("CF")));
                      	match Direction::from(d) {
                        Direction::FromReg => write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware),
                        Direction::ToReg => write_to_address(&reg, &Immediate::UnsignedByte(result as u8), hardware),
                    };
                    (result, overflow)
                     }
                     0b011 => {
                     	let (result, _) = i8::from(value_1).overflowing_add(i8::from(value_2));
                      	let (result, overflow) = result.overflowing_add(i8::from(hardware.clone().read_flags("CF")));
                      	match Direction::from(d) {
                        Direction::FromReg => write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware),
                        Direction::ToReg => write_to_address(&reg, &Immediate::UnsignedByte(result as u8), hardware),
                    };
                    (result, overflow)
                     }
                     0b111 => {
                     	let (result, _) = i8::from(value_1).overflowing_add(i8::from(value_2));
                      	let (result, overflow) = result.overflowing_add(i8::from(hardware.clone().read_flags("CF")));
                      	match Direction::from(d) {
                        Direction::FromReg => write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware),
                        Direction::ToReg => write_to_address(&reg, &Immediate::UnsignedByte(result as u8), hardware),
                    };
                    (result, overflow)
                     }
                     _ => return panic!("Instruction decode error"),
                 }
                 (Immediate::UnsignedWord(value_1), Immediate::UnsignedWord(value_2)) =>{}
                    _ => unreachable!(),
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate to Accumulator
        0b00000100 | 0b00000101 | 0b00010100 | 0b00010101 | 0b00101100 | 0b00101101
        | 0b00011100 | 0b00011101 | 0b00111100 | 0b00111101 => {
            let instruction = match (binary_data[0] & 0b00111100) >> 2 {
                0b0001 => ADD,
                0b0101 => ADC,
                0b1011 => SUB,
                0b0111 => SSB,
                0b1111 => CMP,
                _ => return panic!("Instruction decode error"),
            };
            if let (l, Some(immediate)) = decode_data(w == 0b1, false, &binary_data[1..]) {
                (
                    1 + l,
                    Some(Instruction::ImmediateToAddress(
                        instruction,
                        Addressing::RegisterAddressing(
                            Register::decode(w == 0b1, true, 0b000).unwrap(),
                        ),
                        immediate,
                    )),
                )
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate to Register/Memory
        0b10000000..=0b10000011 => {
            let s = (binary_data[0] & 0b00000010) >> 1;
            if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                if let (dl, Some(immediate)) = decode_data(
                    (w == 0b1) & (s != 0b1),
                    (s == 0b1) & ((binary_data[2 + rl] as i8) < 0i8),
                    &binary_data[(2 + rl)..],
                ) {
                    let instruction = match binary_data[1] & 0b00111000 {
                        0b00000000 => ADD,
                        0b00010000 => ADC,
                        0b00101000 => SUB,
                        0b00011000 => SSB,
                        0b00111000 if w == 0b1 => CMP,
                        0b00111000 if w == 0b0 => CMPBYTE,
                        _ => return panic!("Instruction decode error"),
                    };
                    (
                        2 + rl + dl,
                        Some(Instruction::ImmediateToAddress(instruction, r_m, immediate)),
                    )
                } else {
                    panic!("Instruction decode error")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_increase_decrease_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // Register/Memory
        0b11111110 | 0b11111111 => {
            let w = binary_data[0] & 0b00000001;
            if let (_, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                if (binary_data[1] & 0b00111000) == 0b00000000 {
                    let result = match read_from_address(w == 0b1, &r_m, hardware)
                        .expect("Unable to retrieve data")
                    {
                        Immediate::UnsignedWord(imme) => {
                            let (result, overflow) = i16::from(imme).overflowing_add(1i16);
                            hardware.write_flags("ZF", result == 0);
                            hardware.write_flags("SF", result < 0);
                            hardware.write_flags("OF", overflow);
                            Immediate::UnsignedWord(result as u16)
                        }
                        Immediate::UnsignedByte(imme) => {
                            let (result, overflow) = i8::from(imme).overflowing_add(1i8);
                            hardware.write_flags("ZF", result == 0);
                            hardware.write_flags("SF", result < 0);
                            hardware.write_flags("OF", overflow);
                            Immediate::UnsignedByte(result as u8)
                        }
                        _ => unreachable!(),
                    };
                    write_to_address(&r_m, &result, hardware);
                } else {
                    let result = match read_from_address(w == 0b1, &r_m, hardware)
                        .unwrap_or(panic!("cannot retrieve data"))
                    {
                        Immediate::UnsignedWord(imme) => {
                            let (result, overflow) = i16::from(imme).overflowing_sub(1i16);
                            hardware.write_flags("ZF", result == 0);
                            hardware.write_flags("SF", result < 0);
                            hardware.write_flags("OF", overflow);
                            Immediate::UnsignedWord(result as u16)
                        }
                        Immediate::UnsignedByte(imme) => {
                            let (result, overflow) = i8::from(imme).overflowing_sub(1i8);
                            hardware.write_flags("ZF", result == 0);
                            hardware.write_flags("SF", result < 0);
                            hardware.write_flags("OF", overflow);
                            Immediate::UnsignedByte(result as u8)
                        }
                        _ => unreachable!(),
                    };
                    write_to_address(&r_m, &result, hardware);
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        // Register
        0b01000000..=0b01000111 | 0b01001000..=0b01001111 => {
            if let (_, Some(reg), None) = Addressing::decode(0b1, binary_data, 0b00000111) {
                if (binary_data[0] & 0b11111000) == 0b01000000 {
                    let imme = match read_from_address(true, &reg, hardware)
                        .unwrap_or(panic!("cannot retrieve data"))
                    {
                        Immediate::UnsignedWord(imme) => i16::from(imme),
                        _ => unreachable!(),
                    };
                    let (result, overflow) = imme.overflowing_add(1i16);
                    write_to_address(&reg, &Immediate::UnsignedWord(result as u16), hardware);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", result < 0);
                    hardware.write_flags("OF", overflow);
                } else {
                    let imme = match read_from_address(true, &reg, hardware)
                        .unwrap_or(panic!("cannot retrieve data"))
                    {
                        Immediate::UnsignedWord(imme) => i16::from(imme),
                        _ => unreachable!(),
                    };
                    let (result, overflow) = imme.overflowing_sub(1i16);
                    write_to_address(&reg, &Immediate::UnsignedWord(result as u16), hardware);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", result < 0);
                    hardware.write_flags("OF", overflow);
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_negation_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    if let (_, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
        let result = match read_from_address(w == 0b1, &r_m, hardware)
            .unwrap_or(panic!("cannot retrieve data"))
        {
            Immediate::UnsignedWord(imme) => {
                let (result, overflow) = i16::from(imme).overflowing_neg();
                hardware.write_flags("ZF", result == 0);
                hardware.write_flags("SF", result < 0);
                hardware.write_flags("OF", overflow);
                hardware.write_flags("CF", imme != 0);
                Immediate::UnsignedWord(result as u16)
            }
            Immediate::UnsignedByte(imme) => {
                let (result, overflow) = i8::from(imme).overflowing_neg();
                hardware.write_flags("ZF", result == 0);
                hardware.write_flags("SF", result < 0);
                hardware.write_flags("OF", overflow);
                hardware.write_flags("CF", imme != 0);
                Immediate::UnsignedByte(result as u8)
            }
            _ => unreachable!(),
        };
        write_to_address(&r_m, &result, hardware);
    } else {
        panic!("Instruction decode error")
    }
}

pub fn execute_multiply_divide_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    if binary_data[0] == 0b11110110 | 0b11110111 {
        if let (_, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
            let source = read_from_address(w == 0b1, &r_m, hardware)
                .unwrap_or(panic!("Unable retrieve data"));
            match binary_data[1] & 0b00111000 {
                0b00100000 => {
                    let operand = read_from_address(
                        w == 0b1,
                        Addressing::RegisterAddressing(
                            Register::decode(w == 0b1, true, 0b000).unwrap(),
                        ),
                        hardware,
                    )
                    .unwrap_or(panic!("Unable retrieve data"));
                    match (operand, source) {
                        (Immediate::UnsignedByte(operand), Immediate::UnsignedByte(source)) => {
                            let (result, _) = i16::from(operand).overflowing_mul(i16::from(source));
                            hardware.ax = result as u16;
                            let overflow = (hardware.ax >> 8) != 0;
                            hardware.write_flags("OF", overflow);
                            hardware.write_flags("CF", overflow);
                        }
                        (Immediate::UnsignedWord(operand), Immediate::UnsignedWord(source)) => {
                            let (result, _) = i32::from(operand).overflowing_mul(i32::from(source));
                            hardware.dx = ((result >> 16) & 0xFFFF) as u16;
                            hardware.ax = (result & 0xFFFF) as u16;
                            let overflow = hardware.dx != 0;
                            hardware.write_flags("OF", overflow);
                            hardware.write_flags("CF", overflow);
                        }
                        _ => unreachable!(),
                    }
                }
                0b00101000 => todo!("IMUL"),
                0b00110000 => match source {
                    Immediate::UnsignedByte(source) => {
                        let source = i16::from(source);
                        let operand = i16::from(hardware.ax);
                        let quotient = operand.checked_div(source).unwrap_or(panic!("overflow"));
                        let remainder = operand.checked_rem(source).unwrap_or(panic!("overflow"));
                        hardware.write_to_byte_register(AL, u8::from((quotient as u16) & 0xff));
                        hardware.write_to_byte_register(AH, u8::from((remainder as u16) & 0xff));
                    }
                    Immediate::UnsignedWord(source) => {
                        let source = i32::from(source);
                        let operand = i32::from(hardware.dx) << 16 + i32::from(hardware.ax);
                        let quotient = operand.checked_div(source).unwrap_or(panic!("overflow"));
                        let remainder = operand.checked_rem(source).unwrap_or(panic!("overflow"));
                        hardware.write_to_word_register(AX, u16::from((quotient as u32) & 0xffff));
                        hardware.write_to_word_register(DX, u16::from((remainder as u32) & 0xffff));
                    }
                    _ => unreachable!(),
                },
                0b00111000 => todo!("IDIV"),
                _ => panic!("Instruction decode error"),
            }
        } else {
            panic!("Instruction decode error")
        }
    } else {
        panic!("Instruction decode error")
    }
}

pub fn execute_not_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    if binary_data[0] == 0b11110110 | 0b11110111 {
        if let (_, None, Some(r_m)) = Addressing::decode(w == 0b1, &binary_data[1..], 0b11000111) {
            if (binary_data[1] & 0b00111000) == 0b00011000 {
                match read_from_address(w == 0b1, &r_m, hardware)
                    .unwrap_or(panic!("Unable retrieve data"))
                {
                    Immediate::UnsignedByte(imme) => {
                        write_to_address(&r_m, &Immediate::UnsignedByte(!imme), hardware)
                    }
                    Immediate::UnsignedWord(imme) => {
                        write_to_address(&r_m, &Immediate::UnsignedWord(!imme), hardware)
                    }
                    _ => unreachable!(),
                }
            } else {
                panic!("Instruction decode error")
            }
        } else {
            panic!("Instruction decode error")
        }
    } else {
        panic!("Instruction decode error")
    }
}

pub fn execute_shift_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let v = (binary_data[0] & 0b00000010) >> 1;
    let w = binary_data[0] & 0b00000001;
    if let (_, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
        let source = read_from_address(w == 0b1, &r_m, hardware)
            .expect("Unable to retrieve data");
        let offset = if v == 0b1 {
            hardware.read_from_byte_register(CL) & 0x1f
        } else {
            1u8
        };
        match binary_data[1] & 0b00111000 {
            0b00100000 => match source {
                Immediate::UnsignedByte(source) => {
                    let (result, _) = source.overflowing_shl((offset - 1) as u32);
                    let carry = (result & 0x80) >> 7;
                    let (result, _) = source.overflowing_shl(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i8::from(result) < 0);
                    hardware.write_flags("OF", ((result & 0x80) >> 7) != carry);
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedByte(result), hardware);
                }
                Immediate::UnsignedWord(source) => {
                    let (result, _) = source.overflowing_shl((offset - 1) as u32);
                    let carry = (result & 0x8000) >> 15;
                    let (result, _) = source.overflowing_shl(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i16::from(result) < 0);
                    hardware.write_flags("OF", ((result & 0x8000) >> 15) != carry);
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedWord(result), hardware);
                }
                _ => unreachable!(),
            },
            0b00101000 => match source {
                Immediate::UnsignedByte(source) => {
                    let (result, _) = source.overflowing_shr((offset - 1) as u32);
                    let carry = result & 0x01;
                    let (result, _) = source.overflowing_shr(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i8::from(result) < 0);
                    if v == 0b0 {
                        hardware.write_flags("OF", ((result & 0x80) >> 7) == 0b1);
                    }
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedByte(result), hardware);
                }
                Immediate::UnsignedWord(source) => {
                    let (result, _) = source.overflowing_shr((offset - 1) as u32);
                    let carry = result & 0x0001;
                    let (result, _) = source.overflowing_shr(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i16::from(result) < 0);
                    if v == 0b0 {
                        hardware.write_flags("OF", ((result & 0x8000) >> 15) == 0b1);
                    }
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedWord(result), hardware);
                }
                _ => unreachable!(),
            },
            0b00111000 => match source {
                Immediate::UnsignedByte(source) => {
                    let source = i8::from(source);
                    let (result, _) = source.overflowing_shr((offset - 1) as u32);
                    let carry = (result as u8) & 0x01;
                    let (result, _) = source.overflowing_shr(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i8::from(result) < 0);
                    if v == 0b0 {
                        hardware.write_flags("OF", false);
                    }
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedByte(result as u8), hardware);
                }
                Immediate::UnsignedWord(source) => {
                    let source = i16::from(source);
                    let (result, _) = source.overflowing_shr((offset - 1) as u32);
                    let carry = (result as u16) & 0x0001;
                    let (result, _) = source.overflowing_shr(1);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i16::from(result) < 0);
                    if v == 0b0 {
                        hardware.write_flags("OF", false);
                    }
                    hardware.write_flags("CF", result == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedWord(result as u16), hardware);
                }
                _ => unreachable!(),
            },
            0b00000000 => todo!("ROL"),
            0b00001000 => todo!("ROR"),
            0b00010000 => match source {
                Immediate::UnsignedByte(source) => {
                    let (result, _) = source.overflowing_shl((offset - 1) as u32);
                    let carry = (result & 0x80) >> 7;
                    let (result, _) = source.overflowing_shl(1);
                    result = if hardware.read_flags("CF") {
                        result | 0x01
                    } else {
                        result & 0xfe
                    };
                    if v == 0b0 {
                        hardware.write_flags("OF", (((result & 0x80) >> 7) ^ carry) == 0b1);
                    }
                    hardware.write_flags("CF", carry == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedByte(result), hardware);
                }
                Immediate::UnsignedWord(source) => {
                    let (result, _) = source.overflowing_shl((offset - 1) as u32);
                    let carry = (result & 0x8000) >> 15;
                    let (result, _) = source.overflowing_shl(1);
                    result = if hardware.read_flags("CF") {
                        result | 0x0001
                    } else {
                        result & 0xfffe
                    };
                    if v == 0b0 {
                        hardware.write_flags("OF", (((result & 0x8000) >> 15) ^ carry) == 0b1);
                    }
                    hardware.write_flags("CF", carry == 0b1);
                    write_to_address(&r_m, &Immediate::UnsignedWord(result), hardware);
                }
                _ => unreachable!(),
            },
            0b00011000 => todo!("RCR"),
            _ => return panic!("Instruction decode error"),
        };
    } else {
        panic!("Instruction decode error")
    }
}

pub fn execute_logic_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    match binary_data[0] {
        // Reg./Memory and Register to Either
        0b00100000..=0b00100011 | 0b00001000..=0b00001011 | 0b00110000..=0b00110011 => {
            let d = (binary_data[0] & 0b00000010) >> 1;
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11111111)
            {
                let value_1 = read_from_address(w == 0b1, &reg, hardware)
                    .expect("Unable to retrieve data");
                let value_2 = read_from_address(w == 0b1, &r_m, hardware)
                    .expect("Unable to retrieve data");
                match (value_1, value_2) {
                    (Immediate::UnsignedByte(value_1), Immediate::UnsignedByte(value_2)) => {
                        let result = match binary_data[0] & 0b11111100 {
                            0b00100000 => value_1 & value_2,
                            0b00001000 => value_1 | value_2,
                            0b00110000 => value_1 ^ value_2,
                            _ => panic!("Instruction decode error"),
                        };
                        match Direction::from(d) {
                            Direction::FromReg => {
                                write_to_address(&r_m, &&Immediate::UnsignedByte(result), hardware)
                            }
                            Direction::ToReg => {
                                write_to_address(&reg, &&Immediate::UnsignedByte(result), hardware)
                            }
                        };
                        hardware.write_flags("ZF", result == 0);
                        hardware.write_flags("SF", i8::from(result) < 0);
                    }
                    (Immediate::UnsignedWord(value_1), Immediate::UnsignedWord(value_2)) => {
                        let result = match binary_data[0] & 0b11111100 {
                            0b00100000 => value_1 & value_2,
                            0b00001000 => value_1 | value_2,
                            0b00110000 => value_1 ^ value_2,
                            _ => panic!("Instruction decode error"),
                        };
                        match Direction::from(d) {
                            Direction::FromReg => {
                                write_to_address(&r_m, &Immediate::UnsignedWord(result), hardware)
                            }
                            Direction::ToReg => {
                                write_to_address(&reg, &Immediate::UnsignedWord(result), hardware)
                            }
                        };
                        hardware.write_flags("ZF", result == 0);
                        hardware.write_flags("SF", i16::from(result) < 0);
                    }
                    _ => unreachable!(),
                };
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate to Accumulator
        0b00100100 | 0b00100101 | 0b00001100 | 0b00001101 | 0b00110100 | 0b00110101 => {
            let value_1 = read_from_address(
                w == 0b1,
                Addressing::RegisterAddressing(Register::decode(w == 0b1, true, 0b000).unwrap()),
                hardware,
            )
            .expect("Unable to retrieve data");
            let (_, value_2) = decode_data(w == 0b1, true, &binary_data[1..]);
            let value_2 = value_2.expect("Unable to retrieve data");
            match (value_1, value_2) {
                (Immediate::UnsignedByte(value_1), Immediate::UnsignedByte(value_2)) => {
                    let result = match binary_data[0] & 0b11111110 {
                        0b00100100 => value_1 & value_2,
                        0b00001100 => value_1 | value_2,
                        0b00110100 => value_1 ^ value_2,
                        _ => panic!("Instruction decode error"),
                    };
                    hardware.write_to_byte_register(AL, result);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i8::from(result) < 0);
                }
                (Immediate::UnsignedWord(value_1), Immediate::UnsignedWord(value_2)) => {
                    let result = match binary_data[0] & 0b11111110 {
                        0b00100100 => value_1 & value_2,
                        0b00001100 => value_1 | value_2,
                        0b00110100 => value_1 ^ value_2,
                        _ => panic!("Instruction decode error"),
                    };
                    hardware.write_to_word_register(AX, result);
                    hardware.write_flags("ZF", result == 0);
                    hardware.write_flags("SF", i16::from(result) < 0);
                }
                _ => unreachable!(),
            };
        }
        // Immediate to Register/Memory
        0b10000000..=0b10000011 => {
            if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                let value_1 = read_from_address(w == 0b1, &r_m, hardware)
                    .expect("Unable to retrieve data");
                let (_, value_2) = decode_data(w == 0b1, false, &binary_data[(2 + rl)..]);
                let value_2 = value_2.expect("Unable to retrieve data");
                match (value_1, value_2) {
                    (Immediate::UnsignedByte(value_1), Immediate::UnsignedByte(value_2)) => {
                        let result = match binary_data[1] & 0b00111000 {
                            0b00100000 => value_1 & value_2,
                            0b00001000 => value_1 | value_2,
                            0b00110000 => value_1 ^ value_2,
                            _ => panic!("Instruction decode error"),
                        };
                        write_to_address(&r_m, &Immediate::UnsignedByte(result), hardware);
                        hardware.write_flags("ZF", result == 0);
                        hardware.write_flags("SF", i8::from(result) < 0);
                    }
                    (Immediate::UnsignedWord(value_1), Immediate::UnsignedWord(value_2)) => {
                        let result = match binary_data[1] & 0b00111000 {
                            0b00100000 => value_1 & value_2,
                            0b00001000 => value_1 | value_2,
                            0b00110000 => value_1 ^ value_2,
                            _ => panic!("Instruction decode error"),
                        };
                        write_to_address(&r_m, &Immediate::UnsignedWord(result), hardware);
                        hardware.write_flags("ZF", result == 0);
                        hardware.write_flags("SF", i16::from(result) < 0);
                    }
                    _ => unreachable!(),
                };
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    };
    hardware.write_flags("OF", false);
    hardware.write_flags("CF", false);
}

pub fn execute_test_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    let (value_1, value_2) = match binary_data[0] & 0b11111110 {
        // Register/Memory and Register
        0b10000100 | 0b10000101 => {
            if let (_, Some(reg), Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11111111)
            {
                let value_1 = read_from_address(w == 0b1, &reg, hardware)
                    .expect("Unable to retrieve data");
                let value_2 = read_from_address(w == 0b1, &r_m, hardware)
                    .expect("Unable to retrieve data");
                (value_1, value_2)
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate Data and Register/Memory
        0b11110110 | 0b11110111 if (binary_data[1] & 0b00111000) == 0b0 => {
            if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                let value_1 = read_from_address(w == 0b1, &r_m, hardware)
                    .expect("Unable to retrieve data");
                let (_, value_2) = decode_data(w == 0b1, false, &binary_data[(2 + rl)..]);
                (
                    value_1,
                    value_2.expect("Unable to retrieve data"),
                )
            } else {
                panic!("Instruction decode error")
            }
        }
        // Immediate Data and Accumulator
        0b10101000 | 0b10101001 => {
            let value_1 = read_from_address(
                w == 0b1,
                &Addressing::RegisterAddressing(Register::decode(w == 0b1, true, 0b000).unwrap()),
                hardware,
            )
            .expect("Unable to retrieve data");
            let (_, value_2) = decode_data(w == 0b1, false, &binary_data[1..]);
            (
                value_1,
                value_2.expect("Unable to retrieve data"),
            );
        }
        _ => panic!("Instruction decode error"),
    };
    match (value_1, value_2) {
        (Immediate::UnsignedByte(value_1), Immediate::UnsignedByte(value_2)) => {
            let result = (value_1 & value_2) as i8;
            hardware.write_flags("ZF", result == 0);
            hardware.write_flags("SF", result < 0);
        }
        (Immediate::UnsignedWord(value_1), Immediate::UnsignedWord(value_2)) => {
            let result = (value_1 & value_2) as i16;
            hardware.write_flags("ZF", result == 0);
            hardware.write_flags("SF", result < 0);
        }
        _ => unreachable!(),
    };
    hardware.write_flags("OF", false);
    hardware.write_flags("CF", false);
}

pub fn execute_string_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let w = binary_data[0] & 0b00000001;
    match binary_data[0] {
        0b10100100 | 0b10100101 => {
            let source_address = hardware.ds << 4 + hardware.si;
            let target_address = hardware.es << 4 + hardware.di;
            if w == 0b0 {
                hardware.write_byte_to_memory(
                    target_address,
                    hardware
                        .clone()
                        .read_byte_from_memory(source_address)
                        .expect("Unable to retrieve data"),
                );
                let offset = if hardware.read_flags("DF") { -0b1 } else { 0b1 };
                hardware.si += offset;
                hardware.di += offset;
            } else {
                hardware.write_word_to_memory(
                    target_address,
                    hardware
                        .clone()
                        .read_word_from_memory(source_address)
                        .expect("Unable to retrieve data"),
                );
                let offset = if hardware.read_flags("DF") {
                    -0b10
                } else {
                    0b10
                };
                hardware.si += offset;
                hardware.di += offset;
            }
        }
        0b10100110 | 0b10100111 => todo!("CMPS"),
        0b10101110 | 0b10101111 => todo!("SCAS"),
        0b10101100 | 0b10101101 => todo!("LODSW"),
        0b10101010 | 0b10101011 => todo!("STOSW"),
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_repeat_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        0b11110010 => {
            while hardware.cx != 0 {
                execute_string_instruction(&binary_data[1..], hardware);
                hardware.cx -= 1;
            }
        }
        0b11110011 => {
            while (hardware.cx != 0) | hardware.clone().read_flags("ZF") {
                execute_string_instruction(&binary_data[1..], hardware);
                hardware.cx -= 1;
            }
        }
        _ => return panic!("Instruction decode error"),
    };
}

pub fn execute_jump_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // CALL / JMP (Direct within Segment)
        0b11101000 | 0b11101001 => {
            if binary_data.len() < 3 {
                return panic!("Instruction decode error");
            }
            let immediate = u16::from_le_bytes((&binary_data[1..3]).try_into().unwrap());
            hardware.ip = match binary_data[2] & 0x80 {
                0x0 => hardware.ip + 0x3 + immediate,
                _ => (hardware.ip + 3i16 - ((immediate as i16) * -1i16)) as u16,
            };
        }
        // JMP (Direct within Segment-Short)
        0b11101011 => {
            if binary_data.len() < 2 {
                return panic!("Instruction decode error");
            }
            hardware.ip = match binary_data[1] & 0x80 {
                0x0 => hardware.ip + 0x2 + (binary_data[1] as u16),
                _ => hardware.ip + 0x2 - ((!(binary_data[1] - 0b1)) as u16),
            };
        }
        // CALL / JMP (Indirect within Segment / Indirect Intersegment)
        0b11111111 if match_reg(binary_data[1], &[0b010, 0b011, 0b100, 0b101]) => {
            if let (l, None, Some(r_m)) = Addressing::decode(0b1, &binary_data[1..], 0b11000111) {
                match read_from_address(true, &r_m, hardware)
                    .expect("Unable to retrieve data")
                {
                    Immediate::UnsignedWord(address) => hardware.ip = address,
                    _ => unreachable!(),
                };
            } else {
                panic!("Instruction decode error")
            }
        }
        // Direct Intersegment
        0b10011010 | 0b11101010 => {
            if let (_, Some(offset)) = decode_data(true, false, &binary_data[1..]) {
                if let (_, Some(segment)) = decode_data(true, false, &binary_data[3..]) {
                    hardware.ip = calculate_effective_address(
                        &Addressing::DirectIndexAddressing(offset, segment),
                        hardware,
                    )
                    .expect("Unable to retrieve data");
                } else {
                    panic!("Instruction decode error")
                }
            } else {
                panic!("Instruction decode error")
            }
        }
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_return_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    match binary_data[0] {
        // Within Segment
        0b11000011 => {
            hardware.ip = hardware
                .pop_from_stack()
                .expect("Unable to retrieve data");
        }
        // Within Seg Adding Immed to SP
        0b11000010 => {
            let ip = hardware
                .pop_from_stack()
                .expect("Unable to retrieve data");
            let (_, disp) = decode_data(true, false, &binary_data[1..]);
            hardware.ip = match disp.expect("Unable to retrieve data") {
                Immediate::UnsignedWord(disp) => ((ip as i16) + i16::from(disp)) as u16,
                _ => unreachable!(),
            };
        }
        // Intersegment
        0b11001011 => todo!("RETF"),
        // Intersegment Adding Immediate to SP
        0b11001010 => todo!("RETF"),
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute_conditional_jump_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    let zero_flag = hardware.clone().read_flags("ZF");
    let sign_flag = hardware.clone().read_flags("SF");
    let overflow_flag = hardware.clone().read_flags("OF");
    let carry_flag = hardware.clone().read_flags("CF");
    let jump_signal = match binary_data[0] {
        0b01110100 => zero_flag,
        0b01111100 => sign_flag != overflow_flag,
        0b01111110 => zero_flag || (sign_flag != overflow_flag),
        0b01110010 => carry_flag,
        0b01110110 => carry_flag || zero_flag,
        0b01111010 => todo!("JP"),
        0b01110000 => todo!("JO"),
        0b01111000 => todo!("JS"),
        0b01110101 => !zero_flag,
        0b01111101 => !carry_flag && !zero_flag,
        0b01111111 => !zero_flag && (sign_flag == overflow_flag),
        0b01110011 => !carry_flag,
        0b01110111 => !carry_flag && !zero_flag,
        0b01111011 => todo!("JNP"),
        0b01110001 => todo!("JNO"),
        0b01111001 => todo!("JNS"),
        0b11100010 => {
            hardware.cx -= 1;
            hardware.cx != 0
        }
        0b11100001 => todo!("LOOPZ"),
        0b11100000 => todo!("LOOPNZ"),
        0b11100011 => todo!("JCXZ"),
        _ => return panic!("Instruction decode error"),
    };
    let displacement = match binary_data[1] & 0x80 {
        0b0 => (hardware.ip as i16) + 2i16 + (binary_data[1] as i16),
        _ => (hardware.ip as i16) + 2i16 - (((binary_data[1] as i8) * -1i8) as i16),
    } as u16;
    if jump_signal {
        hardware.ip = displacement;
    }
}

pub fn execute_interrupt_instruction(binary_data: &[u8], hardware: &mut Hardware) {
    hardware.push_to_stack(hardware.clone().read_flags("IF") as u16);
    hardware.write_flags("IF", false);
    match binary_data[0] {
        // Type Specified
        0b11001101 => todo!(),
        // Type 3
        0b11001100 => todo!(),
        _ => panic!("Instruction decode error"),
    }
}

pub fn execute(binary_data: &[u8], hardware: &mut Hardware) {
    if binary_data.is_empty() {
        panic!("data length not enough");
    }
    match binary_data[0] {
        0b11010111 => todo!("XLAT"),
        0b10011111 => todo!("LAHF"),
        0b10011110 => todo!("SAHF"),
        0b10011100 => todo!("PUSHF"),
        0b10011101 => todo!("POPF"),
        0b00110111 => todo!("AAA"),
        0b00100111 => todo!("BAA"),
        0b00111111 => todo!("AAS"),
        0b00101111 => todo!("DAS"),
        0b10011000 => {
            // CBW
            let imme = hardware.clone().read_from_byte_register(AL);
            if (imme & 0x80) == 0x80 {
                hardware.write_to_byte_register(AH, 0xff)
            } else {
                hardware.write_to_byte_register(AH, 0x00)
            }
        }
        0b10011001 => {
            // CWD
            let imme = hardware.clone().read_from_word_register(AX);
            if (imme & 0x8000) == 0x8000 {
                hardware.write_to_word_register(DX, 0xffff)
            } else {
                hardware.write_to_word_register(DX, 0x0000)
            }
        }
        // RET (Within Segment / Intersegment)
        0b11000011 | 0b11001011 => execute_return_instruction(binary_data, hardware),
        // INT (Type 3)
        0b11001100 => execute_interrupt_instruction(binary_data, hardware),
        0b11001110 => todo!("INTO"),
        0b11001111 => todo!("IRET"),
        0b11111000 => todo!("CLC"),
        0b11110101 => todo!("STC"),
        0b11111001 => todo!("STC"),
        0b11111100 => hardware.write_flags("DF", false),
        0b11111101 => hardware.write_flags("DF", true),
        0b11111010 => todo!("CLI"),
        0b11111011 => todo!("STI"),
        0b11110100 => todo!("HLT"),
        0b10011011 => todo!("WAIT"),
        0b11110000 => todo!("LOCK"),
        0b11010101 if binary_data[1] == 0b00001010 => todo!("AAD"),
        0b11010100 if binary_data[1] == 0b00001010 => todo!("AAM"),
        // MOV (Register/Memory to Segment Register / Segment Register to Register/Memory)
        0b10001110 | 0b10001100 if (binary_data[1] & 0b00100000) == 0b0 => {
            execute_move_instruction(binary_data, hardware)
        }
        // PUSH (Register/Memory), CALL / JMP (Indirect within Segment / Indirect Intersegment)
        0b11111111
            if match_reg(
                binary_data[1],
                &[0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b110],
            ) =>
        {
            match (binary_data[1] & 0b00111000) >> 3 {
                0b110 => execute_push_pop_instruction(binary_data, hardware),
                0b010 | 0b011 | 0b100 | 0b101 => execute_jump_instruction(binary_data, hardware),
                0b000 | 0b001 => execute_increase_decrease_instruction(binary_data, hardware),
                _ => panic!("Instruction decode error"),
            }
        }
        // POP (Register/Memory)
        0b10001111 if match_reg(binary_data[1], &[0b000]) => {
            execute_push_pop_instruction(binary_data, hardware)
        }
        // LEA, LDS, LES
        0b10001101 | 0b11000101 | 0b11000100 => execute_load_instruction(binary_data, hardware),
        // CALL (Direct within Segment / Direct Intersegment), JMP (Direct within Segment-Short / Direct within Segment / Direct Intersegment)
        0b11101000 | 0b10011010 | 0b11101001 | 0b11101011 | 0b11101010 => {
            execute_jump_instruction(binary_data, hardware)
        }
        // RET (Within Segment Adding Immediate to SP / Intersegment Adding Immediate to SP)
        0b11000010 | 0b11001010 => execute_return_instruction(binary_data, hardware),
        // conditional jump and loop
        0b01110000..=0b01111111 | 0b11100000..=0b11100011 => {
            execute_conditional_jump_instruction(binary_data, hardware)
        }
        // INT (Type Specified)
        0b11001101 => execute_interrupt_instruction(binary_data, hardware),
        // MOV (Immediate to Register/Memory)
        0b11000110 | 0b11000111 if match_reg(binary_data[1], &[0b000]) => {
            execute_move_instruction(binary_data, hardware)
        }
        // MOV (Register/Memory to/from Register, Immediate to Register, Memory <-> Accumulator )
        0b10001000..=0b10001011
        | 0b10110000..=0b10110111
        | 0b10111000..=0b10111111
        | 0b10100000
        | 0b10100001
        | 0b10100010
        | 0b10100011 => execute_move_instruction(binary_data, hardware),
        // PUSH / POP (Register, Segment Register)
        0b01010000..=0b01010111
        | 0b00000110
        | 0b00001110
        | 0b00010110
        | 0b00011110
        | 0b01011000..=0b01011111
        | 0b00000111
        | 0b00001111
        | 0b00010111
        | 0b00011111 => execute_push_pop_instruction(binary_data, hardware),
        // XCHG
        0b10000110 | 0b10000111 | 0b10010000..=0b10010111 => {
            execute_exchange_instruction(binary_data, hardware)
        }
        // IN / OUT
        0b11100100 | 0b11100101 | 0b11101100 | 0b11101101 | 0b11100110 | 0b11100111
        | 0b11101110 | 0b11101111 => todo!("IN, OUT"),
        // ADD / ADC / SUB / SSB / CMP (Reg./Memory with Register to Either)
        0b00000000..=0b00000011
        | 0b00010000..=0b00010011
        | 0b00101000..=0b00101011
        | 0b00011000..=0b00011011
        | 0b00111000..=0b00111011 => execute_arithmic_instruction(binary_data, hardware),
        // ADD / ADC / SUB / SSB / CMP (Immediate from Register/Memory)
        0b10000000..=0b10000011
            if match_reg(binary_data[1], &[0b000, 0b010, 0b011, 0b101, 0b111]) =>
        {
            execute_arithmic_instruction(binary_data, hardware)
        }
        // ADD / ADC / SUB / SSB / CMP (Immediate to Accumulator)
        0b00000100 | 0b00000101 | 0b00010100 | 0b00010101 | 0b00101100 | 0b00101101
        | 0b00011100 | 0b00011101 | 0b00111100 | 0b00111101 => {
            execute_arithmic_instruction(binary_data, hardware)
        }
        // INC / DEC (Register/Memory)
        0b11111110 if match_reg(binary_data[1], &[0b000, 0b001]) => {
            execute_increase_decrease_instruction(binary_data, hardware)
        }
        // INC / DEC (Register)
        0b01000000..=0b01000111 | 0b01001000..=0b01001111 => {
            execute_increase_decrease_instruction(binary_data, hardware)
        }
        0b11110110 | 0b11110111
            if match_reg(
                binary_data[1],
                &[0b000, 0b010, 0b011, 0b100, 0b101, 0b110, 0b111],
            ) =>
        {
            match (binary_data[1] & 0b00111000) >> 3 {
                // NEG
                0b011 => execute_negation_instruction(binary_data, hardware),
                // MUL, IMUL, DIV, IDIV
                0b100 | 0b101 | 0b110 | 0b111 => {
                    execute_multiply_divide_instruction(binary_data, hardware)
                }
                // NOT
                0b010 => execute_not_instruction(binary_data, hardware),
                // TEST (Immediate Data and Register/Memory)
                0b000 => execute_test_instruction(binary_data, hardware),
                _ => panic!("Instruction decode error"),
            }
        }
        // shift
        0b11010000..=0b11010011
            if match_reg(
                binary_data[1],
                &[0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b111],
            ) =>
        {
            execute_shift_instruction(binary_data, hardware)
        }
        // AND / OR / XOR (Reg./Memory and Register to Either, Immediate to Accumulator )
        0b00100000..=0b00100011
        | 0b00100100
        | 0b00100101
        | 0b00001000..=0b00001011
        | 0b00001100
        | 0b00001101
        | 0b00110000..=0b00110011
        | 0b00110100
        | 0b00110101 => execute_logic_instruction(binary_data, hardware),
        // AND / OR / XOR (Immediate to Register/Memory)
        0b10000000 | 0b10000001 if match_reg(binary_data[1], &[0b001, 0b100, 0b110]) => {
            execute_logic_instruction(binary_data, hardware)
        }
        // TEST (Register/Memory and Register, Immediate Data and Accumulator)
        0b10000100 | 0b10000101 | 0b10101000 | 0b10101001 => {
            execute_test_instruction(binary_data, hardware)
        }
        // REP
        0b11110010 | 0b11110011 => execute_repeat_instruction(binary_data, hardware),
        // ESC
        0b11011000 | 0b11011111 => todo!("ESC"),
        _ => panic!("Instruction decode error"),
    }
}
