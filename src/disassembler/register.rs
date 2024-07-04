use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Register {
    ByteReg(ByteRegister),
    WordReg(WordRegister),
    SegmentReg(SegmentRegister),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Register::ByteReg(register) => write!(f, "{}", register),
            &Register::WordReg(register) => write!(f, "{}", register),
            &Register::SegmentReg(register) => write!(f, "{}", register),
        }
    }
}

impl Register {
    pub fn decode(word_mode: bool, general_register: bool, binary_code: u8) -> Option<Register> {
        match (general_register, word_mode, binary_code) {
            (true, true, 0b000..=0b111) => Some(Register::WordReg(WordRegister::from(binary_code))),
            (true, false, 0b000..=0b111) => {
                Some(Register::ByteReg(ByteRegister::from(binary_code)))
            }
            (false, _, 0b00..=0b11) => {
                Some(Register::SegmentReg(SegmentRegister::from(binary_code)))
            }
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum WordRegister {
    AX = 0b000,
    CX = 0b001,
    DX = 0b010,
    BX = 0b011,
    SP = 0b100,
    BP = 0b101,
    SI = 0b110,
    DI = 0b111,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum ByteRegister {
    AL = 0b000,
    CL = 0b001,
    DL = 0b010,
    BL = 0b011,
    AH = 0b100,
    CH = 0b101,
    DH = 0b110,
    BH = 0b111,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum SegmentRegister {
    ES = 0b00,
    CS = 0b01,
    SS = 0b10,
    DS = 0b11,
}

impl From<u8> for WordRegister {
    fn from(value: u8) -> Self {
        if let 0b000 = value {
            WordRegister::AX
        } else if let 0b001 = value {
            WordRegister::CX
        } else if let 0b010 = value {
            WordRegister::DX
        } else if let 0b011 = value {
            WordRegister::BX
        } else if let 0b100 = value {
            WordRegister::SP
        } else if let 0b101 = value {
            WordRegister::BP
        } else if let 0b110 = value {
            WordRegister::SI
        } else {
            WordRegister::DI
        }
    }
}

impl Display for WordRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let register = match self {
            &WordRegister::AX => "ax",
            &WordRegister::CX => "cx",
            &WordRegister::DX => "dx",
            &WordRegister::BX => "bx",
            &WordRegister::SP => "sp",
            &WordRegister::BP => "bp",
            &WordRegister::SI => "si",
            &WordRegister::DI => "di",
        }
        .to_owned();
        write!(f, "{}", register)
    }
}

impl From<u8> for ByteRegister {
    fn from(value: u8) -> Self {
        if let 0b000 = value {
            ByteRegister::AL
        } else if let 0b001 = value {
            ByteRegister::CL
        } else if let 0b010 = value {
            ByteRegister::DL
        } else if let 0b011 = value {
            ByteRegister::BL
        } else if let 0b100 = value {
            ByteRegister::AH
        } else if let 0b101 = value {
            ByteRegister::CH
        } else if let 0b110 = value {
            ByteRegister::DH
        } else {
            ByteRegister::BH
        }
    }
}

impl Display for ByteRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let register = match self {
            &ByteRegister::AL => "al",
            &ByteRegister::CL => "cl",
            &ByteRegister::DL => "dl",
            &ByteRegister::BL => "bl",
            &ByteRegister::AH => "ah",
            &ByteRegister::CH => "ch",
            &ByteRegister::DH => "dh",
            &ByteRegister::BH => "bh",
        }
        .to_owned();
        write!(f, "{}", register)
    }
}

impl From<u8> for SegmentRegister {
    fn from(value: u8) -> Self {
        if let 0b00 = value {
            SegmentRegister::ES
        } else if let 0b01 = value {
            SegmentRegister::CS
        } else if let 0b10 = value {
            SegmentRegister::SS
        } else {
            SegmentRegister::DS
        }
    }
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let register = match self {
            &SegmentRegister::ES => "es",
            &SegmentRegister::CS => "cs",
            &SegmentRegister::SS => "ss",
            &SegmentRegister::DS => "ds",
        }
        .to_owned();
        write!(f, "{}", register)
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum BaseRegister {
    BX = 0b011,
    BP = 0b101,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum IndexRegister {
    SI = 0b110,
    DI = 0b111,
}

impl Display for BaseRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let register = match self {
            &BaseRegister::BX => "bx",
            &BaseRegister::BP => "bp",
        }
        .to_owned();
        write!(f, "{}", register)
    }
}

impl Display for IndexRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let register = match self {
            &IndexRegister::SI => "si",
            &IndexRegister::DI => "di",
        }
        .to_owned();
        write!(f, "{}", register)
    }
}

#[cfg(test)]
mod tests {
    use crate::disassembler::register::Register;

    #[test]
    fn test_decode_general_word_register() {
        let (word_mode, general_mode) = (true, true);
        let testcases = [
            ("ax", 0b000),
            ("cx", 0b001),
            ("dx", 0b010),
            ("bx", 0b011),
            ("sp", 0b100),
            ("bp", 0b101),
            ("si", 0b110),
            ("di", 0b111),
        ];
        for (i, testcase) in testcases.into_iter().enumerate() {
            if let Some(register) = Register::decode(word_mode, general_mode, testcase.1) {
                assert_eq!(
                    format!("{}", register),
                    testcase.0,
                    "#{}, result: {}, expected: {}",
                    i,
                    register,
                    testcase.0
                )
            }
        }
    }

    #[test]
    fn test_decode_general_byte_register() {
        let (word_mode, general_mode) = (false, true);
        let testcases = [
            ("al", 0b000),
            ("cl", 0b001),
            ("dl", 0b010),
            ("bl", 0b011),
            ("ah", 0b100),
            ("ch", 0b101),
            ("dh", 0b110),
            ("bh", 0b111),
        ];
        for (i, testcase) in testcases.into_iter().enumerate() {
            if let Some(register) = Register::decode(word_mode, general_mode, testcase.1) {
                assert_eq!(
                    format!("{}", register),
                    testcase.0,
                    "#{}, result: {}, expected: {}",
                    i,
                    register,
                    testcase.0
                )
            }
        }
    }

    #[test]
    fn test_decode_segment_register() {
        let (word_mode, general_mode) = (false, false);
        let testcases = [("es", 0b00), ("cs", 0b01), ("ss", 0b10), ("ds", 0b11)];
        for (i, testcase) in testcases.into_iter().enumerate() {
            if let Some(register) = Register::decode(word_mode, general_mode, testcase.1) {
                assert_eq!(
                    format!("{}", register),
                    testcase.0,
                    "#{}, result: {}, expected: {}",
                    i,
                    register,
                    testcase.0
                )
            }
        }
    }
}
