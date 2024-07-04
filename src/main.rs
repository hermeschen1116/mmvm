extern crate core;

mod cli;
mod mmvm;
mod utils;

use crate::mmvm::Disassembler;
use clap::Parser;

use crate::cli::args::Args;
use crate::utils::file_reader::read_file;

fn main() {
    let cli = Args::parse();

    let bytes_data: Vec<u8> = read_file(&cli.d).expect("Failed to read the input file");
    let instructions = Disassembler::disassemble(&bytes_data);
    for (pc, binary, instruction) in instructions.into_iter() {
        let binary = binary
            .iter()
            .map(|&b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");
        println!("{:04x}: {}\t\t{}", pc, binary, instruction);
    }
}
