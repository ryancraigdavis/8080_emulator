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
    //intel_8080_state.init_mem();

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
            // CMA (not) - doesn't affect flags
            0x2f => {
                state.a = !state.a;
            }
            // CMC
            0x3f => {
                state.condition.cy = !state.condition.cy;
            }
            // STC
            0x37 => {
                state.condition.cy = true;
            }
            // ANI
            0xe6 => {
                let x: u8 = state.a & buf[cursor + 1];
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                //Data book says ANI clears CY
                state.condition.cy = false;
                state.a = x;
                state.pc += 1;
            }
            // RRC
            0x0f => {
                let x: u8 = state.a;
                state.a = ((x & 1) << 7) | (x >> 1);
                state.condition.cy = 1 == (x & 1);
            }
            // RAR
            0x1f => {
                let x: u8 = state.a;
                let y: u8 = state.condition.cy as u8;
                state.a = (y << 7) | (x >> 1);
                state.condition.cy = 1 == (x & 1);
            }
            // ADD B
            0x80 => {
                // Increase precision to u16 for ease
                let result: u16 = (state.a as u16) + (state.b as u16);

                // Set flags based on results
                state.condition.set_add_flags(result);

                // Store final value
                state.a = (result as u8) & 0xff;
            }
            // ADD C
            0x81 => {
                let result: u16 = (state.a as u16) + (state.c as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADD D
            0x82 => {
                let result: u16 = (state.a as u16) + (state.d as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADD E
            0x83 => {
                let result: u16 = (state.a as u16) + (state.e as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADD H
            0x84 => {
                let result: u16 = (state.a as u16) + (state.h as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADD L
            0x85 => {
                let result: u16 = (state.a as u16) + (state.l as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADD Mem
            0x86 => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                //let result: u16 = (state.a as u16) + (state.memory[mem_offset as usize] as u16);

                //state.condition.set_add_flags(result);

                //state.a = (result as u8) & 0xff;
            }
            // ADD A
            0x87 => {
                let result: u16 = (state.a as u16) + (state.a as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC B
            0x88 => {
                let result: u16 = (state.a as u16) + (state.b as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC C
            0x89 => {
                let result: u16 = (state.a as u16) + (state.c as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC D
            0x8a => {
                let result: u16 = (state.a as u16) + (state.d as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC E
            0x8b => {
                let result: u16 = (state.a as u16) + (state.e as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC H
            0x8c => {
                let result: u16 = (state.a as u16) + (state.h as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC L
            0x8d => {
                let result: u16 = (state.a as u16) + (state.l as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADC Mem
            0x8e => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                //let result: u16 = (state.a as u16) + (state.memory[mem_offset as usize] as u16) + (state.condition.cy as u16);

                //state.condition.set_add_flags(result);

                //state.a = (result as u8) & 0xff;
            }
            // ADC B
            0x8f => {
                let result: u16 = (state.a as u16) + (state.a as u16) + (state.condition.cy as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
            }
            // ADI byte
            0xc6 => {
                let result: u16 = (state.a as u16) + (buf[cursor + 1] as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
                state.pc += 1;
            }
            // CPI
            0xfe => {
                let x: u8 = state.a - buf[cursor + 1];
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < buf[cursor + 1];
                state.pc += 1;
            }
            // CMP B
            0xb8 => {
                let x: u8 = state.a - state.b;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.b;
                state.pc += 1;
            }
            // CMP C
            0xb9 => {
                let x: u8 = state.a - state.c;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.c;
                state.pc += 1;
            }
            // CMP D
            0xba => {
                let x: u8 = state.a - state.d;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.d;
                state.pc += 1;
            }
            // CMP E
            0xbb => {
                let x: u8 = state.a - state.e;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.e;
                state.pc += 1;
            }
            // CMP H
            0xbc => {
                let x: u8 = state.a - state.h;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.h;
                state.pc += 1;
            }
            // CMP L
            0xbd => {
                let x: u8 = state.a - state.l;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.l;
                state.pc += 1;
            }
            // CMP M
            0xbe => {
                // let x: u8 = state.a - state.m;
                // state.condition.z = x == 0;
                // state.condition.s = 0x80 == (x & 0x80);
                // state.condition.set_parity_flag(x);
                // state.condition.cy = state.a < state.m;
                state.pc += 1;
            }
            // CMP A
            0xbf => {
                let x: u8 = state.a - state.a;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = state.a < state.a;
                state.pc += 1;
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
