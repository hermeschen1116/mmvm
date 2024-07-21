use std::process::exit;

use crate::interpreter::hardware::Hardware;

pub fn execute_systemcall(hardware: &mut Hardware) {
    match hardware.systemcall_count {
        0 => write_systemcall(1, hardware),
        1 => exit_systemcall(),
        _ => todo!(),
    }
}

pub fn write_systemcall(fd: i32, hardware: &mut Hardware) {
    let message_address = hardware.bx;
    let content = "#message";
    println!(
        "<write({}, {:04x}, {}){}=> {}>",
        fd,
        message_address,
        content.len(),
        content,
        content.len()
    );
}

pub fn exit_systemcall() {
    let return_code = 0;
    println!("<exit({})>", return_code);
    exit(0)
}
