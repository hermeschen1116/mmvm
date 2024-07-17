use crate::disassembler::addressing::Addressing;
use crate::disassembler::direction::Direction;
use crate::disassembler::instruction::Instruction;
use crate::disassembler::mnemonic::Mnemonic::*;
use crate::disassembler::numerical::Immediate;
use crate::disassembler::numerical::Numerical;
use crate::disassembler::register::ByteRegister::{AH, AL};
use crate::disassembler::register::WordRegister::{AX, DX};
use crate::interpreter::hardware::Hardware;
use crate::interpreter::utils::*;

pub fn mov(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::AddressToAddress(MOV, direction, reg, r_m) => {
            write_from_address_to_address(direction, &reg, &r_m, hardware)
        }
        &Instruction::ImmediateToAddress(MOV, addr, Numerical::Imme(imme)) => {
            write_to_address(&addr, &imme, hardware)
        }
        &Instruction::ImmediateToAddress(MOVBYTE, addr, Numerical::Imme(imme)) => {
            write_to_address(&addr, &imme, hardware)
        }
        _ => panic!("Unrecognized instruction type: {:?}", instruction),
    }
}

pub fn push(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::WithAddress(PUSH, addr) => {
            if let Some(imme) = read_from_address(true, &addr, hardware) {
                match imme {
                    Immediate::UnsignedWord(imme) => hardware.push_to_stack(imme),
                    _ => unreachable!(),
                }
            } else {
                unreachable!()
            }
        }
        _ => panic!("Unrecognized instruction type: {:?}", instruction),
    }
}

pub fn pop(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::WithAddress(POP, addr) => {
            if let Some(imme) = hardware.pop_from_stack() {
                write_to_address(&addr, &Immediate::UnsignedWord(imme), hardware)
            } else {
                panic!("Not enough data")
            }
        }
        _ => panic!("Unrecognized instruction type: {:?}", instruction),
    }
}

pub fn xchg(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::AddressToAddress(XCHG, direction, reg, r_m) => {
            write_from_address_to_address(direction, &reg, &r_m, hardware)
        }
        _ => panic!("Unrecognized instruction type: {:?}", instruction),
    }
}

pub fn r#in(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn out(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn xlat(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn lea(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::AddressToAddress(LEA, Direction::ToReg, reg, r_m) => {
            write_from_address_to_address(Direction::ToReg, &reg, &r_m, hardware)
        }
        _ => unreachable!(),
    }
}

pub fn lds(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn les(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn lahf(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn sahf(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn pushf(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn popf(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn add(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn adc(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn inc(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn aaa(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn baa(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn sub(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn ssb(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn dec(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn neg(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn cmp(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn aas(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn das(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn mul(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn imul(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn aam(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn div(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn idiv(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn aad(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn cbw(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::Standalone(CBW) => {
            let imme = hardware.clone().read_from_byte_register(AL);
            if (imme & 0x80) == 0x80 {
                hardware.write_to_byte_register(AH, 0xff)
            } else {
                hardware.write_to_byte_register(AH, 0x00)
            }
        }
        _ => todo!(),
    }
}

pub fn cwd(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::Standalone(CWD) => {
            let imme = hardware.clone().read_from_word_register(AX);
            if (imme & 0x8000) == 0x8000 {
                hardware.write_to_word_register(DX, 0xffff)
            } else {
                hardware.write_to_word_register(DX, 0x0000)
            }
        }
        _ => todo!(),
    }
}

pub fn not(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn shl(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn shr(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn sar(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn rol(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn ror(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn rcl(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn rcr(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn and(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn test(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn or(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn xor(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn rep(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn movs(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn cmps(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn scas(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn lods(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn stos(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn call(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jmp(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn ret(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn je(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jl(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jle(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jb(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jbe(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jp(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn jo(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn js(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn jne(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jnl(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jnle(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jnb(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jnbe(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn jnp(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn jno(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn jns(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn l00p(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn loopz(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn loopnz(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn jcxz(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn int(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        _ => todo!(),
    }
}

pub fn into(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn iret(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn clc(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn cmc(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn stc(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn cld(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::Standalone(CLD) => hardware.write_flags("DF", false),
        _ => todo!(),
    }
}

pub fn std(instruction: &Instruction, hardware: &mut Hardware) {
    match instruction {
        &Instruction::Standalone(STD) => hardware.write_flags("DF", true),
        _ => todo!(),
    }
}

pub fn cli(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn sti(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn hlt(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn wait(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn esc(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}

pub fn lock(instruction: &Instruction, hardware: &mut Hardware) {
    todo!()
}
