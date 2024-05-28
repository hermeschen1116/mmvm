use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn read_file(path: &Option<PathBuf>) -> io::Result<Vec<u8>> {
    let mut file: File = File::open(path.as_deref().unwrap())?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
