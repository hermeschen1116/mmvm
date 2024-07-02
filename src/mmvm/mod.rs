use crate::mmvm::instruction::Instruction;

mod addressing;
mod instruction;
mod mnemonic;
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

        let mut pc: usize = 32;

        while pc < binary_code.len() {
            let chunk = &binary_code[pc..];
            if let (length, Some(instruction)) = Instruction::decode(chunk) {
                pc += length;
                instructions.push(instruction);
            } else {
                break;
            }
        }

        instructions
    }
}
