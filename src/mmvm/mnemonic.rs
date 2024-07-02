use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Mnemonic {
    MOV,
    PUSH,
    POP,
    XCHG,
    IN,
    OUT,
    XLAT,
    LEA,
    LDS,
    LES,
    LAHF,
    SAHF,
    PUSHF,
    POPF,
    ADD,
    ADC,
    INC,
    AAA,
    BAA,
    SUB,
    SSB,
    DEC,
    NEG,
    CMP,
    CMP_BYTE,
    AAS,
    DAS,
    MUL,
    IMUL,
    AAM,
    DIV,
    IDIV,
    AAD,
    CBW,
    CWD,
    NOT,
    SHL_S,
    SHL_C,
    SHR_S,
    SHR_C,
    SAR_S,
    SAR_C,
    ROL_S,
    ROL_C,
    ROR_S,
    ROR_C,
    RCL_S,
    RCL_C,
    RCR_S,
    RCR_C,
    AND,
    TEST,
    OR,
    XOR,
    REP_U,
    REP_C,
    MOVSB,
    MOVSW,
    CMPSB,
    CMPSW,
    SCASB,
    SCASW,
    LODSB,
    LODSW,
    STOSB,
    STOSW,
    CALL,
    JMP,
    JMP_S,
    RET_I,
    RET_C,
    JZ,
    JL,
    JLE,
    JB,
    JBE,
    JP,
    JO,
    JS,
    JNZ,
    JNL,
    JNLE,
    JNB,
    JNBE,
    JNP,
    JNO,
    JNS,
    LOOP,
    LOOPZ,
    LOOPNZ,
    JCXZ,
    INT,
    INTO,
    IRET,
    CLC,
    CMC,
    STC,
    CLD,
    STD,
    CLI,
    STI,
    HLT,
    WAIT,
    ESC,
    LOCK,
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mnemonic = match self {
            &Mnemonic::MOV => "mov",
            &Mnemonic::PUSH => "push",
            &Mnemonic::POP => "pop",
            &Mnemonic::XCHG => "xchg",
            &Mnemonic::IN => "in",
            &Mnemonic::OUT => "out",
            &Mnemonic::XLAT => "xlat",
            &Mnemonic::LEA => "lea",
            &Mnemonic::LDS => "lds",
            &Mnemonic::LES => "les",
            &Mnemonic::LAHF => "lahf",
            &Mnemonic::SAHF => "sahf",
            &Mnemonic::PUSHF => "pushf",
            &Mnemonic::POPF => "popf",
            &Mnemonic::ADD => "add",
            &Mnemonic::ADC => "adc",
            &Mnemonic::INC => "inc",
            &Mnemonic::AAA => "aaa",
            &Mnemonic::BAA => "baa",
            &Mnemonic::SUB => "sub",
            &Mnemonic::SSB => "sbb",
            &Mnemonic::DEC => "dec",
            &Mnemonic::NEG => "neg",
            &Mnemonic::CMP => "cmp",
            &Mnemonic::CMP_BYTE => "cmp byte",
            &Mnemonic::AAS => "aas",
            &Mnemonic::DAS => "das",
            &Mnemonic::MUL => "mul",
            &Mnemonic::IMUL => "imul",
            &Mnemonic::AAM => "aam",
            &Mnemonic::DIV => "div",
            &Mnemonic::IDIV => "idiv",
            &Mnemonic::AAD => "aad",
            &Mnemonic::CBW => "cbw",
            &Mnemonic::CWD => "cwd",
            &Mnemonic::NOT => "not",
            &Mnemonic::SHL_S | &Mnemonic::SHL_C => "shl",
            &Mnemonic::SHR_S | &Mnemonic::SHR_C => "shr",
            &Mnemonic::SAR_S | &Mnemonic::SAR_C => "sar",
            &Mnemonic::ROL_S | &Mnemonic::ROL_C => "rol",
            &Mnemonic::ROR_S | &Mnemonic::ROR_C => "ror",
            &Mnemonic::RCL_S | &Mnemonic::RCL_C => "rcl",
            &Mnemonic::RCR_S | &Mnemonic::RCR_C => "rcr",
            &Mnemonic::AND => "and",
            &Mnemonic::TEST => "test",
            &Mnemonic::OR => "or",
            &Mnemonic::XOR => "xor",
            &Mnemonic::REP_U | &Mnemonic::REP_C => "rep",
            &Mnemonic::MOVSB => "movsb",
            &Mnemonic::MOVSW => "movsw",
            &Mnemonic::CMPSB => "cmpsb",
            &Mnemonic::CMPSW => "cmpsw",
            &Mnemonic::SCASB => "scasb",
            &Mnemonic::SCASW => "scasw",
            &Mnemonic::LODSB => "lodsb",
            &Mnemonic::LODSW => "lodsw",
            &Mnemonic::STOSB => "stosb",
            &Mnemonic::STOSW => "stosw",
            &Mnemonic::CALL => "call",
            &Mnemonic::JMP => "jmp",
            &Mnemonic::JMP_S => "jmp short",
            &Mnemonic::RET_I | &Mnemonic::RET_C => "ret",
            &Mnemonic::JZ => "jz",
            &Mnemonic::JL => "jl",
            &Mnemonic::JLE => "jle",
            &Mnemonic::JB => "jb",
            &Mnemonic::JBE => "jbe",
            &Mnemonic::JP => "jp",
            &Mnemonic::JO => "jo",
            &Mnemonic::JS => "js",
            &Mnemonic::JNZ => "jnz",
            &Mnemonic::JNL => "jnl",
            &Mnemonic::JNLE => "jnle",
            &Mnemonic::JNB => "jnb",
            &Mnemonic::JNBE => "jnbe",
            &Mnemonic::JNP => "jnp",
            &Mnemonic::JNO => "jno",
            &Mnemonic::JNS => "jns",
            &Mnemonic::LOOP => "loop",
            &Mnemonic::LOOPZ => "loopz",
            &Mnemonic::LOOPNZ => "loopnz",
            &Mnemonic::JCXZ => "jcxz",
            &Mnemonic::INT => "int",
            &Mnemonic::INTO => "into",
            &Mnemonic::IRET => "iret",
            &Mnemonic::CLC => "clc",
            &Mnemonic::CMC => "cmc",
            &Mnemonic::STC => "stc",
            &Mnemonic::CLD => "cld",
            &Mnemonic::STD => "std",
            &Mnemonic::CLI => "cli",
            &Mnemonic::STI => "sti",
            &Mnemonic::HLT => "hlt",
            &Mnemonic::WAIT => "wait",
            &Mnemonic::ESC => "esc",
            &Mnemonic::LOCK => "lock",
        }
        .to_owned();
        write!(f, "{}", mnemonic)
    }
}
