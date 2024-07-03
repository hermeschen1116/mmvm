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

    pub fn disassemble(&self, binary_code: &[u8]) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = Vec::new();

        let mut pc: u16 = 0x0;

        while (pc as usize) < binary_code.len() {
            let chunk = &binary_code[(pc as usize)..];
            if let (length, Some(instruction)) = Instruction::decode(pc, chunk) {
                pc += length as u16;
                instructions.push(instruction);
            } else {
                break;
            }
        }

        instructions
    }
}
