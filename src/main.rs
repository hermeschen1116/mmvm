extern crate core;

mod cli;
mod mmvm;
mod utils;

use clap::Parser;

use crate::cli::args::Args;
use crate::utils::file_reader::read_file;

fn main() {
    let cli = Args::parse();

    let bytes_data: Vec<u8> = read_file(&cli.d).expect("Failed to read the input file");
    dbg!(bytes_data);
}
