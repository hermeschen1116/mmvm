use crate::mmvm::instruction::Instruction;

mod addressing;
mod instruction;
mod mnemonic;
mod numerical;
mod register;

pub struct Disassembler;

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Disassembler {
    pub fn new() -> Self {
        Disassembler
    }

    pub fn disassemble(binary_code: &[u8]) -> Vec<(u16, &[u8], Instruction)> {
        let mut instructions: Vec<(u16, &[u8], Instruction)> = Vec::new();

        let a_hdrlen = binary_code[4];
        let a_text: u32 = binary_code[8..12]
            .iter()
            .enumerate()
            .map(|(i, &byte)| (byte as u32) << (8 * i))
            .sum();
        let mut pc: usize = 0;
        while pc < (a_text as usize) {
            let chunk = &binary_code[((a_hdrlen as usize) + pc)..];
            if let (length, Some(instruction)) = Instruction::decode(pc as u16, chunk) {
                instructions.push(((pc as u16), &chunk[..length], instruction));
                pc += length;
            } else {
                print!("error: {:?}", chunk);
                break;
            }
        }

        instructions
    }
}
