use clap::Parser;
use std::path::PathBuf;

/// Simple program to disassemble binary files
#[derive(Parser, Debug)]
#[command(name = "mmvm")]
#[command(about = "a disassembler", long_about = None)]
pub struct Args {
    /// Input file to disassemble
    #[arg(short, long, value_name = "FILE")]
    pub d: Option<PathBuf>,

    #[arg(long, action = clap::ArgAction::Count)]
    debug: u8,
}
