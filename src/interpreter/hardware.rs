use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use crate::disassembler::register::{ByteRegister, SegmentRegister, WordRegister};

#[derive(Debug, Clone)]
pub struct Hardware {
    pub ax: u16,
    pub cx: u16,
    pub dx: u16,
    pub bx: u16,
    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,
    pub es: u16,
    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    pub ip: u16,
    flag: u16,
    stack: Vec<u8>,
    memory: HashMap<u16, u8>,
    pub systemcall_count: usize,
}

impl Hardware {
    pub fn new() -> Self {
        Self {
            ax: 0x0000,
            cx: 0x0000,
            dx: 0x0000,
            bx: 0x0000,
            sp: 0x0000,
            bp: 0x0000,
            si: 0x0000,
            di: 0x0000,
            es: 0x0000,
            cs: 0x0000,
            ss: 0x0000,
            ds: 0x0000,
            ip: 0x0000,
            flag: 0x0000,
            stack: Vec::new(),
            memory: HashMap::new(),
            systemcall_count: 0,
        }
    }

    pub fn write_to_word_register(&mut self, reg: WordRegister, value: u16) {
        match reg {
            WordRegister::AX => self.ax = value,
            WordRegister::CX => self.cx = value,
            WordRegister::DX => self.dx = value,
            WordRegister::BX => self.bx = value,
            WordRegister::SP => self.sp = value,
            WordRegister::BP => self.bp = value,
            WordRegister::SI => self.si = value,
            WordRegister::DI => self.di = value,
        }
    }

    pub fn read_from_word_register(self, reg: WordRegister) -> u16 {
        match reg {
            WordRegister::AX => self.ax,
            WordRegister::CX => self.cx,
            WordRegister::DX => self.dx,
            WordRegister::BX => self.bx,
            WordRegister::SP => self.sp,
            WordRegister::BP => self.bp,
            WordRegister::SI => self.si,
            WordRegister::DI => self.di,
        }
    }

    pub fn write_to_byte_register(&mut self, reg: ByteRegister, value: u8) {
        match reg {
            ByteRegister::AL => self.ax = (self.ax & 0xff00) + u16::from(value),
            ByteRegister::CL => self.cx = (self.cx & 0xff00) + u16::from(value),
            ByteRegister::DL => self.dx = (self.dx & 0xff00) + u16::from(value),
            ByteRegister::BL => self.bx = (self.bx & 0xff00) + u16::from(value),
            ByteRegister::AH => self.ax = (self.ax & 0x00ff) + (u16::from(value) << 8),
            ByteRegister::CH => self.cx = (self.cx & 0x00ff) + (u16::from(value) << 8),
            ByteRegister::DH => self.dx = (self.dx & 0x00ff) + (u16::from(value) << 8),
            ByteRegister::BH => self.bx = (self.bx & 0x00ff) + (u16::from(value) << 8),
        }
    }

    pub fn read_from_byte_register(self, reg: ByteRegister) -> u8 {
        match reg {
            ByteRegister::AL => (self.ax & 0x00ff) as u8,
            ByteRegister::CL => (self.cx & 0x00ff) as u8,
            ByteRegister::DL => (self.dx & 0x00ff) as u8,
            ByteRegister::BL => (self.bx & 0x00ff) as u8,
            ByteRegister::AH => (self.ax & 0xff00) as u8,
            ByteRegister::CH => (self.cx & 0xff00) as u8,
            ByteRegister::DH => (self.dx & 0xff00) as u8,
            ByteRegister::BH => (self.bx & 0xff00) as u8,
        }
    }

    pub fn write_to_segment_register(&mut self, reg: SegmentRegister, value: u16) {
        match reg {
            SegmentRegister::ES => self.es = value,
            SegmentRegister::CS => self.cs = value,
            SegmentRegister::SS => self.ss = value,
            SegmentRegister::DS => self.ds = value,
        }
    }

    pub fn read_from_segment_register(self, reg: SegmentRegister) -> u16 {
        match reg {
            SegmentRegister::ES => self.es,
            SegmentRegister::CS => self.cs,
            SegmentRegister::SS => self.ss,
            SegmentRegister::DS => self.ds,
        }
    }

    pub fn push_to_stack(&mut self, value: u16) {
        if self.stack.is_empty() {
            self.sp = 0x0001;
        } else {
            self.sp += 0x0002;
        }
        let [value_low, value_high] = value.to_le_bytes();
        self.stack.push(value_high);
        self.stack.push(value_low);
    }

    pub fn pop_from_stack(&mut self) -> Option<u16> {
        self.sp -= 0x0002;
        if let Some(data_low) = self.stack.pop() {
            if let Some(data_high) = self.stack.pop() {
                Some(u16::from_le_bytes([data_low, data_high]))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn write_byte_to_memory(&mut self, addr: u16, value: u8) {
        self.memory.insert(addr, value);
    }

    pub fn write_word_to_memory(&mut self, addr: u16, value: u16) {
        let [value_low, value_high] = value.to_le_bytes();
        self.memory.insert(addr, value_low);
        self.memory.insert(addr + 0b1, value_high);
    }

    pub fn read_byte_from_memory(self, addr: u16) -> Option<u8> {
        self.memory.get(&addr).copied()
    }

    pub fn read_word_from_memory(self, addr: u16) -> Option<u16> {
        if let Some(value_low) = self.memory.get(&addr).copied() {
            if let Some(value_high) = self.memory.get(&(addr + 0b1)).copied() {
                let value = u16::from_le_bytes([value_low, value_high]);
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn read_flags(self, flag: &str) -> bool {
        match flag.to_uppercase().as_str() {
            "CF" => ((self.flag & 0xfffe) >> 0) == 0b1,
            "PF" => ((self.flag & 0xfffb) >> 2) == 0b1,
            "AF" => ((self.flag & 0xffef) >> 4) == 0b1,
            "ZF" => ((self.flag & 0xffbf) >> 6) == 0b1,
            "SF" => ((self.flag & 0xff7f) >> 7) == 0b1,
            "TF" => ((self.flag & 0xfeff) >> 8) == 0b1,
            "IF" => ((self.flag & 0xfdff) >> 9) == 0b1,
            "DF" => ((self.flag & 0xfbff) >> 10) == 0b1,
            "OF" => ((self.flag & 0xf7ff) >> 11) == 0b1,
            _ => panic!("Unrecognized flag"),
        }
    }

    pub fn write_flags(&mut self, flag: &str, status: bool) {
        self.flag = match flag.to_uppercase().as_str() {
            "CF" => (self.flag & 0xfffe) + ((status as u16) << 0),
            "PF" => (self.flag & 0xfffb) + ((status as u16) << 2),
            "AF" => (self.flag & 0xffef) + ((status as u16) << 4),
            "ZF" => (self.flag & 0xffbf) + ((status as u16) << 6),
            "SF" => (self.flag & 0xff7f) + ((status as u16) << 7),
            "TF" => (self.flag & 0xfeff) + ((status as u16) << 8),
            "IF" => (self.flag & 0xfdff) + ((status as u16) << 9),
            "DF" => (self.flag & 0xfbff) + ((status as u16) << 10),
            "OF" => (self.flag & 0xf7ff) + ((status as u16) << 11),
            _ => panic!("Unrecognized flag"),
        };
    }
}

impl Display for Hardware {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let zf = if self.clone().read_flags("ZF") {
            "Z"
        } else {
            "-"
        };
        let sf = if self.clone().read_flags("SF") {
            "S"
        } else {
            "-"
        };
        let of = if self.clone().read_flags("OF") {
            "O"
        } else {
            "-"
        };
        let cf = if self.clone().read_flags("CF") {
            "C"
        } else {
            "-"
        };
        write!(
            f,
            "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {}{}{}{} {:04x}:",
            self.ax,
            self.bx,
            self.cx,
            self.dx,
            self.sp,
            self.bp,
            self.si,
            self.di,
            zf,
            sf,
            of,
            cf,
            self.ip
        )
    }
}
