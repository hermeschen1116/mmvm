use crate::{disassembler::instruction::Instruction, utils::header::Header};
use std::cmp;

pub mod addressing;
pub mod direction;
pub mod instruction;
pub mod mnemonic;
pub mod numerical;
pub mod register;
mod test;

pub struct Disassembler {
    pub asm: Vec<(u16, Vec<u8>, Instruction)>,
}

impl Disassembler {
    pub fn new() -> Self {
        Self { asm: Vec::new() }
    }

    pub fn disassemble(&mut self, header: Header, bytes_data: &[u8]) {
        let (a_hdrlen, a_text) = (header.a_hdrlen as usize, header.a_text as usize);
        let decode_area = &bytes_data[a_hdrlen..];
        let mut pc: usize = 0;
        loop {
            let chunk = &decode_area[pc..];
            if let (length, Some(instruction)) = Instruction::decode(pc as u16, chunk) {
                match (pc + length).cmp(&a_text) {
                    cmp::Ordering::Less => {
                        self.asm
                            .push(((pc as u16), chunk[..length].to_vec(), instruction));
                        pc += length;
                    }
                    cmp::Ordering::Equal => {
                        self.asm
                            .push(((pc as u16), chunk[..length].to_vec(), instruction));
                        break;
                    }
                    cmp::Ordering::Greater => {
                        self.asm
                            .push(((pc as u16), [0b00].to_vec(), Instruction::Undefined));
                        break;
                    }
                }
            } else {
                panic!("disassembler error, {:?}", chunk);
            }
        }
    }

    pub fn print(&self) {
        if self.asm.is_empty() {
            panic!("you haven't done disassembling")
        }
        for (pc, binary, instruction) in self.asm.clone().into_iter() {
            let binary = binary
                .iter()
                .map(|&b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .concat();
            println!("{:04x}: {}\t{}", pc, binary, instruction);
        }
    }
}
