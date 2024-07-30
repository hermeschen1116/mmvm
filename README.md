# MMVM

## How to compile?

```sh
cargo build --release
```

## How to execute?

```sh
# disassemble
cargo run -r -- -d $binary_file
# interpret
cargo run -r -- -m $binary_file
```

## Architecture

### cli module

- args.rs: Handles command-line arguments parsing. It defines how the program accepts inputs from the user.

### disassembler module

- addressing.rs: Contains logic related to addressing modes in the disassembler.
- direction.rs: Manages the direction flags of the instructions.
- instruction.rs: Defines the structure and parsing of instructions.
- mnemonic.rs: Manages mnemonics, the human-readable names for instructions.
- numerical.rs: Handles numerical type data.
- register.rs: Manages enums represent registers.

### interpreter module

- execution.rs: Handles the execution logic of interpreted instructions.
- hardware.rs: Simulates the hardware components interacting with the interpreter.
- systemcall.rs: Manages system calls within the interpreter.
- utils.rs: Utility functions supporting the interpreter.

### utils module

- file_reader.rs: Contains functionality for reading input files.
- header.rs: Manages file headers or any initial metadata.

### Module Interactions

- cli: Interacts with main.rs to parse and handle command-line inputs.
- disassembler: Works with main.rs to disassemble input data, using various submodules to handle specific parts of the disassembly process.
- interpreter: Also interacts with main.rs, executing instructions and simulating the hardware environment.
- utils: Provides utility functions and file handling capabilities to support other modules.

## Comments on Overall Impression

- There are still some bugs to fix in interpreter
- Still trying to figure out how to simulate hardware and system call
- Need to optimize on how to match byte code pattern
