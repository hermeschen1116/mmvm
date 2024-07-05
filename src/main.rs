extern crate core;

mod cli;
mod disassembler;
mod utils;

use crate::disassembler::Disassembler;
use clap::Parser;
use utils::header::Header;

use crate::cli::args::Args;
use crate::utils::file_reader::read_file;

fn main() {
    let cli = Args::parse();

    let bytes_data: Vec<u8> = read_file(&cli.d).expect("Failed to read the input file");
    let header = Header::new(&bytes_data);

    let mut disassembler = Disassembler::new();
    disassembler.disassemble(header, &bytes_data);

    disassembler.print();
}
