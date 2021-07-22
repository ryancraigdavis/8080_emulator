use std::fs;
use std::io::prelude::*;
mod condition_codes;
mod disassembler;
mod intel8080_state;
use intel8080_state::StateIntel8080;

fn main() {
    let file_name = String::from("invaders");

    let mut buf = Vec::new();
    let mut file_in = fs::File::open(file_name).unwrap();
    file_in.read_to_end(&mut buf).unwrap();

    // If we need to print out the full disassembly
    //disassembler::print_all(&buf);

    let mut intel_8080_state: StateIntel8080 = Default::default();

    // Main emulation function
    run_emulation(&mut intel_8080_state, &buf);

    // Print out the current state (debugging)
    println!("{:?}", intel_8080_state);
}

fn run_emulation(state: &mut StateIntel8080, buf: &Vec<u8>) {
    // Loop control and current instruction location
    let mut run_emu: bool = true;
    let mut cursor: usize;

    while run_emu {
        cursor = state.pc as usize;
        match buf[cursor] {
            // NOP
            0x00 => {}
            // LXI B,word
            0x01 => {
                state.c = buf[cursor + 2];
                state.b = buf[cursor + 1];
                state.pc += 2;
            }
            // MOV B,C
            0x41 => {
                state.b = state.c;
            }
            // MOV B,D
            0x42 => {
                state.b = state.d;
            }
            // MOV B,E
            0x43 => {
                state.b = state.e;
            }
            // Everything else (unimplemented)
            _ => {
                run_emu = unimplemented(&buf[cursor]);
            }
        }

        state.pc += 1;
    }
}

fn unimplemented(hexcode: &u8) -> bool {
    // If the instruction isn't implemented yet
    println!("Unimplemented instruction : {:02x}", hexcode);
    false
}
