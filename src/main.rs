#![feature(bigint_helper_methods)]
extern crate core;

mod cli;
mod disassembler;
mod interpreter;
mod utils;

use crate::disassembler::Disassembler;
use crate::interpreter::Interpreter;
use crate::utils::header::Header;
use clap::Parser;

use crate::cli::args::Args;
use crate::utils::file_reader::read_file;

fn main() {
    let cli = Args::parse();

    let binary_path = if cli.d.is_some() {
        cli.d.clone()
    } else {
        cli.m.clone()
    };
    let bytes_data: Vec<u8> = read_file(&binary_path).expect("Failed to read the input file");
    let header = Header::new(&bytes_data);

    let mut disassembler = Disassembler::new();
    disassembler.disassemble(header.clone(), &bytes_data);

    let mut interpreter = Interpreter::new();

    if cli.d.is_some() {
        disassembler.print();
    } else {
        interpreter.execute(header.clone(), &bytes_data, disassembler.asm);
    }
}
