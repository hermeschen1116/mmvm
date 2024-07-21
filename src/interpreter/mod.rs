use execution::execute;
use hardware::Hardware;

use crate::disassembler::instruction::Instruction;
use crate::utils::header::Header;

mod execution;
mod hardware;
mod systemcall;
mod utils;

pub struct Interpreter {
    pub hardware: Hardware,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            hardware: Hardware::new(),
        }
    }

    pub fn execute(
        &mut self,
        header: Header,
        bytes_data: &[u8],
        instructions: Vec<(u16, Vec<u8>, Instruction)>,
    ) {
        let (a_hdrlen, a_text) = (header.a_hdrlen as usize, header.a_text as usize);
        let text_area = &bytes_data[a_hdrlen..];
        let data_area = &bytes_data[(a_hdrlen + a_text)..];
        for (addr, &byte) in data_area.into_iter().enumerate() {
            self.hardware.write_byte_to_memory(addr as u16, byte)
        }

        println!(" AX   BX   CX   DX   SP   BP   SI   DI  FLAG  IP");
        for (length, binary, instruction) in instructions.into_iter() {
            let ip = self.hardware.ip as usize;
            let chunk = &text_area[ip..];
            execute(chunk, &mut self.hardware);

            let binary = binary
                .iter()
                .map(|&b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .concat();
            println!("{}{}\t\t{}", self.hardware, binary, instruction);
            self.hardware.ip += length as u16;
        }
    }
}
