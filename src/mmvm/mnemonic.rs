use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum Mnemonic {
    MOV,
    MOVBYTE,
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
    CMPBYTE,
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
    SHL,
    SHR,
    SAR,
    ROL,
    ROR,
    RCL,
    RCR,
    AND,
    TEST,
    TESTBYTE,
    OR,
    XOR,
    REP,
    REPNE,
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
    JMPSHORT,
    RET,
    RETF,
    JE,
    JL,
    JLE,
    JB,
    JBE,
    JP,
    JO,
    JS,
    JNE,
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
            &Mnemonic::MOVBYTE => "mov byte",
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
            &Mnemonic::CMPBYTE => "cmp byte",
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
            &Mnemonic::SHL => "shl",
            &Mnemonic::SHR => "shr",
            &Mnemonic::SAR => "sar",
            &Mnemonic::ROL => "rol",
            &Mnemonic::ROR => "ror",
            &Mnemonic::RCL => "rcl",
            &Mnemonic::RCR => "rcr",
            &Mnemonic::AND => "and",
            &Mnemonic::TEST => "test",
            &Mnemonic::TESTBYTE => "test byte",
            &Mnemonic::OR => "or",
            &Mnemonic::XOR => "xor",
            &Mnemonic::REP => "rep",
            &Mnemonic::REPNE => "repne",
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
            &Mnemonic::JMPSHORT => "jmp short",
            &Mnemonic::RET => "ret",
            &Mnemonic::RETF => "retf",
            &Mnemonic::JE => "je",
            &Mnemonic::JL => "jl",
            &Mnemonic::JLE => "jle",
            &Mnemonic::JB => "jb",
            &Mnemonic::JBE => "jbe",
            &Mnemonic::JP => "jp",
            &Mnemonic::JO => "jo",
            &Mnemonic::JS => "js",
            &Mnemonic::JNE => "jne",
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
