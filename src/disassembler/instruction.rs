use std::fmt::{Display, Formatter};

use crate::disassembler::addressing::Addressing;
use crate::disassembler::direction::Direction;
use crate::disassembler::mnemonic::Mnemonic;
use crate::disassembler::mnemonic::Mnemonic::*;
use crate::disassembler::numerical::{Immediate, Numerical};
use crate::disassembler::register::ByteRegister::CL;
use crate::disassembler::register::Register;
use crate::disassembler::register::WordRegister::{AX, DX};

fn match_reg(binary_data: u8, reference: &[u8]) -> bool {
    let reg = (binary_data & 0b00111000) >> 3;
    reference.contains(&reg)
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Standalone(Mnemonic),
    WithInstruction(Mnemonic, Mnemonic),
    WithAddress(Mnemonic, Addressing),
    AddressToAddress(Mnemonic, Direction, Addressing, Addressing),
    WithImmediate(Mnemonic, Numerical),
    ImmediateToAddress(Mnemonic, Addressing, Numerical),
    Undefined,
}

impl Instruction {
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

    pub fn decode_move_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // Register/Memory to/from Register
            0b10001000..=0b10001011 => {
                let d = (binary_data[0] & 0b00000010) >> 1;
                let w = binary_data[0] & 0b00000001;
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(w, &binary_data[1..], 0b11111111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            MOV,
                            Direction::from(d),
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate to Register/Memory
            0b11000110 | 0b11000111 if match_reg(binary_data[1], &[0b000]) => {
                let w = binary_data[0] & 0b00000001;
                let instruction = if w == 0b1 { MOV } else { MOVBYTE };
                if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111)
                {
                    if let (dl, Some(immediate)) =
                        Self::decode_data(w == 0b1, false, &binary_data[(2 + rl)..])
                    {
                        (
                            2 + rl + dl,
                            Some(Instruction::ImmediateToAddress(instruction, r_m, immediate)),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate to Register
            0b10110000..=0b10111111 => {
                let w = (binary_data[0] & 0b00001000) >> 3;
                if let (_, Some(reg), None) = Addressing::decode(w, binary_data, 0b00000111) {
                    if let (dl, Some(immediate)) =
                        Self::decode_data(w == 0b1, false, &binary_data[1..])
                    {
                        (
                            1 + dl,
                            Some(Instruction::ImmediateToAddress(MOV, reg, immediate)),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Memory <-> Accumulator
            0b10100000 | 0b10100001 | 0b10100010 | 0b10100011 => {
                let d = (binary_data[0] & 0b00000010) >> 1;
                let w = binary_data[0] & 0b00000001;
                if let (_, Some(address)) = Self::decode_data(true, false, &binary_data[1..]) {
                    (
                        3,
                        Some(Instruction::AddressToAddress(
                            MOV,
                            Direction::from(d),
                            Addressing::RegisterAddressing(
                                Register::decode(w == 0b1, true, 0b000).unwrap(),
                            ),
                            Addressing::DirectAddressing(address),
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Register/Memory <-> Segment Register
            0b10001110 | 0b10001100 if (binary_data[1] & 0b00100000) == 0b00000000 => {
                let d = (binary_data[0] & 0b00000010) >> 1;
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(0, &binary_data[1..], 0b11011111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            MOV,
                            Direction::from(d),
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_push_pop_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // PUSH (Register/Memory)
            0b11111111 if (binary_data[1] & 0b00111000) == 0b00110000 => {
                if let (l, None, Some(r_m)) = Addressing::decode(0, &binary_data[1..], 0b11000111) {
                    (2 + l, Some(Instruction::WithAddress(PUSH, r_m)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // PUSH (Register)
            0b01010000..=0b01010111 => {
                if let (l, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                    (1 + l, Some(Instruction::WithAddress(PUSH, reg)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // POP (Register/Memory)
            0b10001111 if (binary_data[1] & 0b00111000) == 0b00000000 => {
                if let (l, None, Some(r_m)) = Addressing::decode(0, &binary_data[1..], 0b11000111) {
                    (2 + l, Some(Instruction::WithAddress(POP, r_m)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // POP (Register)
            0b01011000..=0b01011111 => {
                if let (l, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                    (1 + l, Some(Instruction::WithAddress(POP, reg)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // PUSH / POP (Segment Register)
            _ => {
                let instruction = if (binary_data[0] & 0b11100111) == 0b00000110 {
                    PUSH
                } else if (binary_data[0] & 0b11100111) == 0b00000111 {
                    POP
                } else {
                    return (0, Some(Instruction::Undefined));
                };
                if let (l, Some(reg), None) = Addressing::decode(0, &binary_data[1..], 0b00011000) {
                    (1 + l, Some(Instruction::WithAddress(instruction, reg)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
        }
    }

    pub fn decode_exchange_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // Register/Memory with Register
            0b10000110 | 0b10000111 => {
                let w = binary_data[0] & 0b00000001;
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(w, &binary_data[1..], 0b11111111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            XCHG,
                            Direction::FromReg,
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Register with Accumulator
            0b10010000..=0b10010111 => {
                if let (l, Some(reg), None) = Addressing::decode(0, binary_data, 0b00000111) {
                    (
                        1 + l,
                        Some(Instruction::AddressToAddress(
                            XCHG,
                            Direction::FromReg,
                            Addressing::RegisterAddressing(Register::WordReg(AX)),
                            reg,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_in_out_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let w = binary_data[0] & 0b00000001;
        let instruction = match binary_data[0] & 0b00000010 {
            0b00 => IN,
            0b10 => OUT,
            _ => return (0, Some(Instruction::Undefined)),
        };
        match binary_data[0] {
            // Fixed Port
            0b11100100 | 0b11100101 => (
                2,
                Some(Instruction::ImmediateToAddress(
                    instruction,
                    Addressing::RegisterAddressing(
                        Register::decode(w == 0b1, true, 0b000).unwrap(),
                    ),
                    Numerical::Imme(Immediate::from(&binary_data[1..2], false)),
                )),
            ),
            // Variable Port
            0b11101100 | 0b11101101 => (
                1,
                Some(Instruction::AddressToAddress(
                    instruction,
                    Direction::FromReg,
                    Addressing::RegisterAddressing(Register::WordReg(DX)),
                    Addressing::RegisterAddressing(
                        Register::decode(w == 0b1, true, 0b000).unwrap(),
                    ),
                )),
            ),
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_load_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let instruction = match binary_data[0] {
            0b10001101 => LEA,
            0b11000101 => LDS,
            0b11000100 => LES,
            _ => return (0, Some(Instruction::Undefined)),
        };
        if let (l, Some(reg), Some(r_m)) = Addressing::decode(0b1, &binary_data[1..], 0b11111111) {
            (
                2 + l,
                Some(Instruction::AddressToAddress(
                    instruction,
                    Direction::ToReg,
                    reg,
                    r_m,
                )),
            )
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_arithmic_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let w = binary_data[0] & 0b00000001;
        match binary_data[0] {
            // Reg./Memory with Register to Either
            0b00000000..=0b00000011
            | 0b00010000..=0b00010011
            | 0b00101000..=0b00101011
            | 0b00011000..=0b00011011
            | 0b00111000..=0b00111011 => {
                let instruction = match (binary_data[0] & 0b00111000) >> 3 {
                    0b000 => ADD,
                    0b010 => ADC,
                    0b101 => SUB,
                    0b011 => SSB,
                    0b111 => CMP,
                    _ => return (0, Some(Instruction::Undefined)),
                };
                let d = (binary_data[0] & 0b00000010) >> 1;
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(w, &binary_data[1..], 0b11111111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            instruction,
                            Direction::from(d),
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
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
                    _ => return (0, Some(Instruction::Undefined)),
                };
                if let (l, Some(immediate)) = Self::decode_data(w == 0b1, false, &binary_data[1..])
                {
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
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate to Register/Memory
            0b10000000..=0b10000011 => {
                let s = (binary_data[0] & 0b00000010) >> 1;
                if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111)
                {
                    if let (dl, Some(immediate)) = Self::decode_data(
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
                            _ => return (0, Some(Instruction::Undefined)),
                        };
                        (
                            2 + rl + dl,
                            Some(Instruction::ImmediateToAddress(instruction, r_m, immediate)),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_increase_decrease_instruction(
        binary_data: &[u8],
    ) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // Register/Memory
            0b11111110 | 0b11111111 => {
                let w = binary_data[0] & 0b00000001;
                if let (l, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
                    match binary_data[1] & 0b00111000 {
                        0b00000000 => (2 + l, Some(Instruction::WithAddress(INC, r_m))),
                        0b00001000 => (2 + l, Some(Instruction::WithAddress(DEC, r_m))),
                        _ => (0, Some(Instruction::Undefined)),
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Register
            0b01000000..=0b01000111 | 0b01001000..=0b01001111 => {
                let instruction = if (binary_data[0] & 0b11111000) == 0b01000000 {
                    INC
                } else {
                    DEC
                };
                if let (l, Some(reg), None) = Addressing::decode(0b1, binary_data, 0b00000111) {
                    (1 + l, Some(Instruction::WithAddress(instruction, reg)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_negation_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        if let (l, None, Some(r_m)) =
            Addressing::decode(binary_data[0] & 0b1, &binary_data[1..], 0b11000111)
        {
            (2 + l, Some(Instruction::WithAddress(NEG, r_m)))
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_multiply_divide_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        if binary_data[0] == 0b11110110 | 0b11110111 {
            if let (l, None, Some(r_m)) =
                Addressing::decode(binary_data[0] & 0b1, &binary_data[1..], 0b11000111)
            {
                match binary_data[1] & 0b00111000 {
                    0b00100000 => (2 + l, Some(Instruction::WithAddress(MUL, r_m))),
                    0b00101000 => (2 + l, Some(Instruction::WithAddress(IMUL, r_m))),
                    0b00110000 => (2 + l, Some(Instruction::WithAddress(DIV, r_m))),
                    0b00111000 => (2 + l, Some(Instruction::WithAddress(IDIV, r_m))),
                    _ => (0, Some(Instruction::Undefined)),
                }
            } else {
                (0, Some(Instruction::Undefined))
            }
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_not_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        if binary_data[0] == 0b11110110 | 0b11110111 {
            if let (l, None, Some(r_m)) =
                Addressing::decode(binary_data[0] & 0b1, &binary_data[1..], 0b11000111)
            {
                if (binary_data[1] & 0b00111000) == 0b00011000 {
                    (2 + l, Some(Instruction::WithAddress(NOT, r_m)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            } else {
                (0, Some(Instruction::Undefined))
            }
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_shift_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let v = (binary_data[0] & 0b00000010) >> 1;
        let w = binary_data[0] & 0b00000001;

        let instruction = match binary_data[1] & 0b00111000 {
            0b00100000 => SHL,
            0b00101000 => SHR,
            0b00111000 => SAR,
            0b00000000 => ROL,
            0b00001000 => ROR,
            0b00010000 => RCL,
            0b00011000 => RCR,
            _ => return (0, Some(Instruction::Undefined)),
        };

        if let (l, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111) {
            if v == 0b1 {
                (
                    2 + l,
                    Some(Instruction::AddressToAddress(
                        instruction,
                        Direction::FromReg,
                        Addressing::RegisterAddressing(Register::ByteReg(CL)),
                        r_m,
                    )),
                )
            } else {
                (
                    2 + l,
                    Some(Instruction::ImmediateToAddress(
                        instruction,
                        r_m,
                        Numerical::Imme(Immediate::UnsignedByte(0b1)),
                    )),
                )
            }
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_logic_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let w = binary_data[0] & 0b00000001;
        match binary_data[0] {
            // Reg./Memory and Register to Either
            0b00100000..=0b00100011 | 0b00001000..=0b00001011 | 0b00110000..=0b00110011 => {
                let instruction = match binary_data[0] & 0b11111100 {
                    0b00100000 => AND,
                    0b00001000 => OR,
                    0b00110000 => XOR,
                    _ => return (0, Some(Instruction::Undefined)),
                };
                let d = (binary_data[0] & 0b00000010) >> 1;
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(w, &binary_data[1..], 0b11111111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            instruction,
                            Direction::from(d),
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate to Accumulator
            0b00100100 | 0b00100101 | 0b00001100 | 0b00001101 | 0b00110100 | 0b00110101 => {
                let instruction = match binary_data[0] & 0b11111110 {
                    0b00100100 => AND,
                    0b00001100 => OR,
                    0b00110100 => XOR,
                    _ => return (0, Some(Instruction::Undefined)),
                };
                if let (l, Some(immediate)) = Self::decode_data(w == 0b1, true, &binary_data[1..]) {
                    (
                        2 + l,
                        Some(Instruction::ImmediateToAddress(
                            instruction,
                            Addressing::RegisterAddressing(
                                Register::decode(w == 0b1, true, 0b000).unwrap(),
                            ),
                            immediate,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate to Register/Memory
            0b10000000..=0b10000011 => {
                if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111)
                {
                    if let (dl, Some(immediate)) =
                        Self::decode_data(w == 0b1, false, &binary_data[(2 + rl)..])
                    {
                        let instruction = match binary_data[1] & 0b00111000 {
                            0b00100000 => AND,
                            0b00001000 => OR,
                            0b00110000 => XOR,
                            _ => return (0, Some(Instruction::Undefined)),
                        };
                        (
                            2 + rl + dl,
                            Some(Instruction::ImmediateToAddress(instruction, r_m, immediate)),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_test_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let w = binary_data[0] & 0b00000001;
        let instruction = match (w == 0b1) | ((binary_data[1] & 0b11000000) == 0b11000000) {
            true => TEST,
            false => TESTBYTE,
        };
        match binary_data[0] & 0b11111110 {
            // Register/Memory and Register
            0b10000100 | 0b10000101 => {
                if let (l, Some(reg), Some(r_m)) =
                    Addressing::decode(w, &binary_data[1..], 0b11111111)
                {
                    (
                        2 + l,
                        Some(Instruction::AddressToAddress(
                            instruction,
                            Direction::FromReg,
                            reg,
                            r_m,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate Data and Register/Memory
            0b11110110 | 0b11110111 if (binary_data[1] & 0b00111000) == 0b0 => {
                if let (rl, None, Some(r_m)) = Addressing::decode(w, &binary_data[1..], 0b11000111)
                {
                    if let (dl, Some(immediate)) =
                        Self::decode_data(w == 0b1, false, &binary_data[(2 + rl)..])
                    {
                        (
                            2 + rl + dl,
                            Some(Instruction::ImmediateToAddress(instruction, r_m, immediate)),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Immediate Data and Accumulator
            0b10101000 | 0b10101001 => {
                if let (l, Some(immediate)) = Self::decode_data(w == 0b1, false, &binary_data[1..])
                {
                    (
                        1 + l,
                        Some(Instruction::ImmediateToAddress(
                            TEST,
                            Addressing::RegisterAddressing(
                                Register::decode(w == 0b1, true, 0b000).unwrap(),
                            ),
                            immediate,
                        )),
                    )
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_string_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // MOVS (Byte)
            0b10100100 => (1, Some(Instruction::Standalone(MOVSB))),
            // MOVS (Word)
            0b10100101 => (1, Some(Instruction::Standalone(MOVSW))),
            // CMPS (Byte)
            0b10100110 => (1, Some(Instruction::Standalone(CMPSB))),
            // CMPS (Word)
            0b10100111 => (1, Some(Instruction::Standalone(CMPSW))),
            // SCAS (Byte)
            0b10101110 => (1, Some(Instruction::Standalone(SCASB))),
            // SCAS (Word)
            0b10101111 => (1, Some(Instruction::Standalone(SCASW))),
            // LODS (Byte)
            0b10101100 => (1, Some(Instruction::Standalone(LODSB))),
            // LODS (Word)
            0b10101101 => (1, Some(Instruction::Standalone(LODSW))),
            // STOS (Byte)
            0b10101010 => (1, Some(Instruction::Standalone(STOSB))),
            // STOS (Word)
            0b10101011 => (1, Some(Instruction::Standalone(STOSW))),
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_repeat_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        let instruction = match binary_data[0] {
            0b11110010 => REP,
            0b11110011 => REPNE,
            _ => return (0, Some(Instruction::Undefined)),
        };
        if let (l, Some(Instruction::Standalone(sub_instruction))) =
            Self::decode_string_instruction(&binary_data[1..])
        {
            (
                1 + l,
                Some(Instruction::WithInstruction(instruction, sub_instruction)),
            )
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode_jump_instruction(pc: u16, binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // CALL / JMP (Direct within Segment)
            0b11101000 | 0b11101001 => {
                let instruction = match binary_data[0] & 0b1 {
                    0b0 => CALL,
                    0b1 => JMP,
                    _ => return (0, Some(Instruction::Undefined)),
                };
                if binary_data.len() < 3 {
                    return (0, Some(Instruction::Undefined));
                }
                let immediate = u16::from_le_bytes((&binary_data[1..3]).try_into().unwrap());
                let displacement = match binary_data[2] & 0x80 {
                    0x0 => (pc as u16) + 0x3 + immediate,
                    _ => ((pc as i16) + 3i16 - ((immediate as i16) * -1i16)) as u16,
                };
                (
                    3,
                    Some(Instruction::WithImmediate(
                        instruction,
                        Numerical::Imme(Immediate::UnsignedWord(displacement)),
                    )),
                )
            }
            // JMP (Direct within Segment-Short)
            0b11101011 => {
                if binary_data.len() < 2 {
                    return (0, Some(Instruction::Undefined));
                }
                let displacement = match binary_data[1] & 0x80 {
                    0x0 => (pc as u16) + 0x2 + (binary_data[1] as u16),
                    _ => (pc as u16) + 0x2 - ((!(binary_data[1] - 0b1)) as u16),
                };
                (
                    2,
                    Some(Instruction::WithImmediate(
                        JMPSHORT,
                        Numerical::Imme(Immediate::UnsignedWord(displacement)),
                    )),
                )
            }
            // CALL / JMP (Indirect within Segment / Indirect Intersegment)
            0b11111111 if match_reg(binary_data[1], &[0b010, 0b011, 0b100, 0b101]) => {
                let instruction = match (binary_data[1] & 0b00111000) >> 3 {
                    0b010 | 0b011 => CALL,
                    0b100 | 0b101 => JMP,
                    _ => return (0, Some(Instruction::Undefined)),
                };
                if let (l, None, Some(r_m)) = Addressing::decode(0b1, &binary_data[1..], 0b11000111)
                {
                    (2 + l, Some(Instruction::WithAddress(instruction, r_m)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Direct Intersegment
            0b10011010 | 0b11101010 => {
                let instruction = if binary_data[0] == 0b10011010 {
                    CALL
                } else {
                    JMP
                };
                if let (_, Some(offset)) = Self::decode_data(true, false, &binary_data[1..]) {
                    if let (_, Some(segment)) = Self::decode_data(true, false, &binary_data[3..]) {
                        (
                            5,
                            Some(Instruction::WithAddress(
                                instruction,
                                Addressing::DirectIndexAddressing(offset, segment),
                            )),
                        )
                    } else {
                        (0, Some(Instruction::Undefined))
                    }
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_return_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // Within Segment
            0b11000011 => (1, Some(Instruction::Standalone(RET))),
            // Within Seg Adding Immed to SP
            0b11000010 => {
                if let (_, Some(data)) = Self::decode_data(true, false, &binary_data[1..]) {
                    (3, Some(Instruction::WithImmediate(RET, data)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            // Intersegment
            0b11001011 => (1, Some(Instruction::Standalone(RETF))),
            // Intersegment Adding Immediate to SP
            0b11001010 => {
                if let (_, Some(data)) = Self::decode_data(true, true, &binary_data[1..]) {
                    (3, Some(Instruction::WithImmediate(RETF, data)))
                } else {
                    (0, Some(Instruction::Undefined))
                }
            }
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_conditional_jump_instruction(
        pc: u16,
        binary_data: &[u8],
    ) -> (usize, Option<Instruction>) {
        let instruction = match binary_data[0] {
            0b01110100 => JE,
            0b01111100 => JL,
            0b01111110 => JLE,
            0b01110010 => JB,
            0b01110110 => JBE,
            0b01111010 => JP,
            0b01110000 => JO,
            0b01111000 => JS,
            0b01110101 => JNE,
            0b01111101 => JNL,
            0b01111111 => JNLE,
            0b01110011 => JNB,
            0b01110111 => JNBE,
            0b01111011 => JNP,
            0b01110001 => JNO,
            0b01111001 => JNS,
            0b11100010 => LOOP,
            0b11100001 => LOOPZ,
            0b11100000 => LOOPNZ,
            0b11100011 => JCXZ,
            _ => return (0, Some(Instruction::Undefined)),
        };
        let displacement = match binary_data[1] & 0x80 {
            0b0 => (pc as i16) + 2i16 + (binary_data[1] as i16),
            _ => (pc as i16) + 2i16 - (((binary_data[1] as i8) * -1i8) as i16),
        } as u16;
        (
            2,
            Some(Instruction::WithImmediate(
                instruction,
                Numerical::Imme(Immediate::UnsignedWord(displacement)),
            )),
        )
    }

    pub fn decode_interrupt_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        match binary_data[0] {
            // Type Specified
            0b11001101 => (
                2,
                Some(Instruction::WithImmediate(
                    INT,
                    Numerical::Imme(Immediate::UnsignedByte(binary_data[1])),
                )),
            ),
            // Type 3
            0b11001100 => (1, Some(Instruction::Standalone(INT))),
            _ => (0, Some(Instruction::Undefined)),
        }
    }

    pub fn decode_escape_instruction(binary_data: &[u8]) -> (usize, Option<Instruction>) {
        if let (l, None, Some(r_m)) = Addressing::decode(0b1, &binary_data[1..], 0b11000111) {
            (2 + l, Some(Instruction::WithAddress(ESC, r_m)))
        } else {
            (0, Some(Instruction::Undefined))
        }
    }

    pub fn decode(pc: u16, binary_data: &[u8]) -> (usize, Option<Instruction>) {
        if binary_data.is_empty() {
            panic!("data length not enough");
        }
        match binary_data[0] {
            0b11010111 => (1, Some(Instruction::Standalone(XLAT))),
            0b10011111 => (1, Some(Instruction::Standalone(LAHF))),
            0b10011110 => (1, Some(Instruction::Standalone(SAHF))),
            0b10011100 => (1, Some(Instruction::Standalone(PUSHF))),
            0b10011101 => (1, Some(Instruction::Standalone(POPF))),
            0b00110111 => (1, Some(Instruction::Standalone(AAA))),
            0b00100111 => (1, Some(Instruction::Standalone(BAA))),
            0b00111111 => (1, Some(Instruction::Standalone(AAS))),
            0b00101111 => (1, Some(Instruction::Standalone(DAS))),
            0b10011000 => (1, Some(Instruction::Standalone(CBW))),
            0b10011001 => (1, Some(Instruction::Standalone(CWD))),
            // RET (Within Segment / Intersegment)
            0b11000011 | 0b11001011 => Self::decode_return_instruction(binary_data),
            // INT (Type 3)
            0b11001100 => Self::decode_interrupt_instruction(binary_data),
            0b11001110 => (1, Some(Instruction::Standalone(INTO))),
            0b11001111 => (1, Some(Instruction::Standalone(IRET))),
            0b11111000 => (1, Some(Instruction::Standalone(CLC))),
            0b11110101 => (1, Some(Instruction::Standalone(CMC))),
            0b11111001 => (1, Some(Instruction::Standalone(STC))),
            0b11111100 => (1, Some(Instruction::Standalone(CLD))),
            0b11111101 => (1, Some(Instruction::Standalone(STD))),
            0b11111010 => (1, Some(Instruction::Standalone(CLI))),
            0b11111011 => (1, Some(Instruction::Standalone(STI))),
            0b11110100 => (1, Some(Instruction::Standalone(HLT))),
            0b10011011 => (1, Some(Instruction::Standalone(WAIT))),
            0b11110000 => (1, Some(Instruction::Standalone(LOCK))),
            0b11010101 if binary_data[1] == 0b00001010 => (2, Some(Instruction::Standalone(AAD))),
            0b11010100 if binary_data[1] == 0b00001010 => (2, Some(Instruction::Standalone(AAM))),
            // MOV (Register/Memory to Segment Register / Segment Register to Register/Memory)
            0b10001110 | 0b10001100 if (binary_data[1] & 0b00100000) == 0b0 => {
                Self::decode_move_instruction(binary_data)
            }
            // PUSH (Register/Memory), CALL / JMP (Indirect within Segment / Indirect Intersegment)
            0b11111111
                if match_reg(
                    binary_data[1],
                    &[0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b110],
                ) =>
            {
                match (binary_data[1] & 0b00111000) >> 3 {
                    0b110 => Self::decode_push_pop_instruction(binary_data),
                    0b010 | 0b011 | 0b100 | 0b101 => Self::decode_jump_instruction(pc, binary_data),
                    0b000 | 0b001 => Self::decode_increase_decrease_instruction(binary_data),
                    _ => (0, Some(Instruction::Undefined)),
                }
            }
            // POP (Register/Memory)
            0b10001111 if match_reg(binary_data[1], &[0b000]) => {
                Self::decode_push_pop_instruction(binary_data)
            }
            // LEA, LDS, LES
            0b10001101 | 0b11000101 | 0b11000100 => Self::decode_load_instruction(binary_data),
            // CALL (Direct within Segment / Direct Intersegment), JMP (Direct within Segment-Short / Direct within Segment / Direct Intersegment)
            0b11101000 | 0b10011010 | 0b11101001 | 0b11101011 | 0b11101010 => {
                Self::decode_jump_instruction(pc, binary_data)
            }
            // RET (Within Segment Adding Immediate to SP / Intersegment Adding Immediate to SP)
            0b11000010 | 0b11001010 => Self::decode_return_instruction(binary_data),
            // conditional jump and loop
            0b01110000..=0b01111111 | 0b11100000..=0b11100011 => {
                Self::decode_conditional_jump_instruction(pc, binary_data)
            }
            // INT (Type Specified)
            0b11001101 => Self::decode_interrupt_instruction(binary_data),
            // MOV (Immediate to Register/Memory)
            0b11000110 | 0b11000111 if match_reg(binary_data[1], &[0b000]) => {
                Self::decode_move_instruction(binary_data)
            }
            // MOV (Register/Memory to/from Register, Immediate to Register, Memory <-> Accumulator )
            0b10001000..=0b10001011
            | 0b10110000..=0b10110111
            | 0b10111000..=0b10111111
            | 0b10100000
            | 0b10100001
            | 0b10100010
            | 0b10100011 => Self::decode_move_instruction(binary_data),
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
            | 0b00011111 => Self::decode_push_pop_instruction(binary_data),
            // XCHG
            0b10000110 | 0b10000111 | 0b10010000..=0b10010111 => {
                Self::decode_exchange_instruction(binary_data)
            }
            // IN / OUT
            0b11100100 | 0b11100101 | 0b11101100 | 0b11101101 | 0b11100110 | 0b11100111
            | 0b11101110 | 0b11101111 => Self::decode_in_out_instruction(binary_data),
            // ADD / ADC / SUB / SSB / CMP (Reg./Memory with Register to Either)
            0b00000000..=0b00000011
            | 0b00010000..=0b00010011
            | 0b00101000..=0b00101011
            | 0b00011000..=0b00011011
            | 0b00111000..=0b00111011 => Self::decode_arithmic_instruction(binary_data),
            // ADD / ADC / SUB / SSB / CMP (Immediate from Register/Memory)
            0b10000000..=0b10000011
                if match_reg(binary_data[1], &[0b000, 0b010, 0b011, 0b101, 0b111]) =>
            {
                Self::decode_arithmic_instruction(binary_data)
            }
            // ADD / ADC / SUB / SSB / CMP (Immediate to Accumulator)
            0b00000100 | 0b00000101 | 0b00010100 | 0b00010101 | 0b00101100 | 0b00101101
            | 0b00011100 | 0b00011101 | 0b00111100 | 0b00111101 => {
                Self::decode_arithmic_instruction(binary_data)
            }
            // INC / DEC (Register/Memory)
            0b11111110 if match_reg(binary_data[1], &[0b000, 0b001]) => {
                Self::decode_increase_decrease_instruction(binary_data)
            }
            // INC / DEC (Register)
            0b01000000..=0b01000111 | 0b01001000..=0b01001111 => {
                Self::decode_increase_decrease_instruction(binary_data)
            }
            0b11110110 | 0b11110111
                if match_reg(
                    binary_data[1],
                    &[0b000, 0b010, 0b011, 0b100, 0b101, 0b110, 0b111],
                ) =>
            {
                match (binary_data[1] & 0b00111000) >> 3 {
                    // NEG
                    0b011 => Self::decode_negation_instruction(binary_data),
                    // MUL, IMUL, DIV, IDIV
                    0b100 | 0b101 | 0b110 | 0b111 => {
                        Self::decode_multiply_divide_instruction(binary_data)
                    }
                    // NOT
                    0b010 => Self::decode_not_instruction(binary_data),
                    // TEST (Immediate Data and Register/Memory)
                    0b000 => Self::decode_test_instruction(binary_data),
                    _ => (0, Some(Instruction::Undefined)),
                }
            }
            // shift
            0b11010000..=0b11010011
                if match_reg(
                    binary_data[1],
                    &[0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b111],
                ) =>
            {
                Self::decode_shift_instruction(binary_data)
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
            | 0b00110101 => Self::decode_logic_instruction(binary_data),
            // AND / OR / XOR (Immediate to Register/Memory)
            0b10000000 | 0b10000001 if match_reg(binary_data[1], &[0b001, 0b100, 0b110]) => {
                Self::decode_logic_instruction(binary_data)
            }
            // TEST (Register/Memory and Register, Immediate Data and Accumulator)
            0b10000100 | 0b10000101 | 0b10101000 | 0b10101001 => {
                Self::decode_test_instruction(binary_data)
            }
            // string manipulation
            0b10100100 | 0b10100101 | 0b10100110 | 0b10100111 | 0b10101110 | 0b10101111
            | 0b10101100 | 0b10101101 | 0b10101010 | 0b10101011 => {
                Self::decode_string_instruction(binary_data)
            }
            // REP
            0b11110010 | 0b11110011 => Self::decode_repeat_instruction(binary_data),
            // ESC
            0b11011000 | 0b11011111 => Self::decode_escape_instruction(binary_data),
            _ => (0, Some(Instruction::Undefined)),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Instruction::Standalone(mnemonic) => write!(f, "{}", mnemonic),
            &Instruction::WithInstruction(mnemonic, sub_mnemonic) => {
                write!(f, "{} {}", mnemonic, sub_mnemonic)
            }
            &Instruction::WithAddress(mnemonic, target) => write!(f, "{} {}", mnemonic, target),
            &Instruction::AddressToAddress(mnemonic, direction, reg, r_m) => match direction {
                Direction::FromReg => write!(f, "{} {}, {}", mnemonic, r_m, reg),
                Direction::ToReg => write!(f, "{} {}, {}", mnemonic, reg, r_m),
            },
            &Instruction::WithImmediate(mnemonic, immediate) => {
                write!(f, "{} {}", mnemonic, immediate)
            }
            &Instruction::ImmediateToAddress(mnemonic, target, immediate) => {
                write!(f, "{} {}, {}", mnemonic, target, immediate)
            }
            &Instruction::Undefined => write!(f, "(undefined)"),
        }
    }
}
