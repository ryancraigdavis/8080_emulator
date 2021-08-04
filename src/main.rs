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

    intel_8080_state.init_mem();

    // Main emulation function
    run_emulation(&mut intel_8080_state, &mut buf);

    // Print out the current state (debugging)
    //println!("{:?}", intel_8080_state);
}

fn run_emulation(state: &mut StateIntel8080, buf: &mut Vec<u8>) {
    // Loop control and current instruction location
    let mut run_emu: bool = true;
    let mut cursor: usize;
    let mut incr: bool = true;
    let mut printstate: bool = false;

    while run_emu {
        incr = true;
        printstate = false;
        cursor = state.pc as usize;
        print!("{:04x} ", cursor);
        disassembler::get_single(&buf, cursor);
        match buf[cursor] {
            // NOP
            0x00 => {}
            // LXI B,word
            0x01 => {
                state.c = buf[cursor + 2];
                state.b = buf[cursor + 1];
                state.pc += 2;
            }
            // INR B
            0x04 => {
                let result: u16 = (state.b as u16) + 1;
                state.condition.set_inr_flags(result);
                state.b = (result as u8) & 0xff;
            }
            // DCR B
            0x05 => {
                let result: u16 = (state.b as u16) - 1;
                state.condition.z = result == 0;
                state.condition.s = 0x80 == (result & 0x80);
                state.condition.set_parity_flag(result as u8);
                state.b = result as u8;
            }
            // DCR C
            0x0d => {
                let result: u16 = (state.c as u16) - 1;
                state.condition.z = result == 0;
                state.condition.s = 0x80 == (result & 0x80);
                state.condition.set_parity_flag(result as u8);
                state.c = result as u8;
            }
            // MVI B,byte
            0x06 => {
                state.b = buf[cursor + 1];
                state.pc += 1;
            }
            // DAD B
            0x09 => {
                let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                let bc: u16 = ((state.b as u16) << 8) | state.c as u16;
                let result: u16 = hl + bc;
                state.h = (result & 0xff00) as u8;
                state.l = (result & 0xff) as u8;
                state.condition.cy = (result & 0xffff) > 0;
            }
            // DAD D
            0x19 => {
                let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                let de: u16 = ((state.d as u16) << 8) | state.e as u16;
                let result: u16 = hl + de;
                state.h = (result & 0xff00) as u8;
                state.l = (result & 0xff) as u8;
                state.condition.cy = (result & 0xffff) != 0;
            }
            // DAD H
            0x29 => {
                let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                let result: u16 = hl + hl;
                state.h = (result & 0xff00) as u8;
                state.l = (result & 0xff) as u8;
                state.condition.cy = (result & 0xffff) != 0;
            }
            // LXI D,word
            0x11 => {
                state.e = buf[cursor + 2];
                state.d = buf[cursor + 1];
                state.pc += 2;
            }
            // LXI H,word
            0x21 => {
                state.h = buf[cursor + 2];
                state.l = buf[cursor + 1];
                state.pc += 2;
                printstate = true;
            }
            // LXI SP,word
            0x31 => {
                state.sp = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.pc += 2;
            }
            // INX D
            0x13 => {
                state.e += 1;
                if state.e == 0 {
                    state.d += 1;
                } 
            }
            // INX H
            0x23 => {
                state.l += 1;
                if state.l == 0 {
                    state.h += 1;
                } 
            }
            // LDAX D
            0x1a => {
                //let mem_offset: u16 = (state.d << 8) | state.e;
                //state.a = state.memory[mem_offset];
            }
            // INR C
            0x0c => {
                let result: u16 = (state.c as u16) + 1;
                state.condition.set_inr_flags(result);
                state.c = (result as u8) & 0xff;
            }
            // MVI C,byte
            0x0e => {
                state.c = buf[cursor + 1];
                state.pc += 1;
            }
            // MVI A,byte
            0x3e => {
                state.a = buf[cursor + 1];
                state.pc += 1;
            }
            // MVI H,byte
            0x26 => {
                state.h = buf[cursor + 1];
                state.pc += 1;
            }
            // INR D
            0x14 => {
                let result: u16 = (state.d as u16) + 1;
                state.condition.set_inr_flags(result);
                state.d = (result as u8) & 0xff;
            }
            // INR E
            0x1c => {
                let result: u16 = (state.e as u16) + 1;
                state.condition.set_inr_flags(result);
                state.e = (result as u8) & 0xff;
            }
            // INR H
            0x24 => {
                let result: u16 = (state.h as u16) + 1;
                state.condition.set_inr_flags(result);
                state.h = (result as u8) & 0xff;
            }
            // INR L
            0x2c => {
                let result: u16 = (state.l as u16) + 1;
                state.condition.set_inr_flags(result);
                state.l = (result as u8) & 0xff;
            }
            // INR Mem
            0x34 => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                //let result: u16 = (state.memory[mem_offset as usize] as u16) + 1;
                //state.condition.set_inr_flags(result);
                //state.memory[mem_offset as usize] = (result as u8) & 0xff;
            }
            // INR A
            0x3c => {
                let result: u16 = (state.a as u16) + 1;
                state.condition.set_inr_flags(result);
                state.a = (result as u8) & 0xff;
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
            // SUB A
            0x97 => {
                // Subtracts A from A, equaling zero
                let result: u16 = 0;
                state.condition.set_sub_flags(result);
                state.condition.cy = true;
                state.a = (result as u8) & 0xff;
            }
            // ADI byte
            0xc6 => {
                let result: u16 = (state.a as u16) + (buf[cursor + 1] as u16);

                state.condition.set_add_flags(result);

                state.a = (result as u8) & 0xff;
                state.pc += 1;
            }
            // ACI byte
            0xce => {
                let result: u16 =
                    (state.a as u16) + (buf[cursor + 1] as u16) + (state.condition.cy as u16);

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

            //new starts here
            //EI
            0xfb => {
                state.condition.set_inr_flags(1);
            }

            //DI
            //return to, unfinished -- wrong opcode?
            0xc6 => {
                state.condition.set_inr_flags(0);
            }

            //in -says to leave unimplemented and return to later
            0xdb => {}

            //out - says to leave unimplemented and return to later
            0xd3 => {}

            //jmp
            0xc3 => {
                state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                incr = false;
            }

            //JNZ
            0xc2 => {
                if state.condition.z {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //JZ
            0xf2 => {
                if !state.condition.z {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //JC
            0xda => {
                if !state.condition.cy {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //JNC
            0xd2 => {
                if state.condition.cy {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //JPO
            0xe2 => {
                if !state.condition.p {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //jpe
            0xea => {
                if state.condition.p {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //jp (plus)
            0xf2 => {
                if state.condition.s {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //jm (minus)
            0xfa => {
                if !state.condition.s {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //call - doesn't implement negative
            //return to this
            0xcd => {
                let result = (state.pc as u16) + 2;
                //buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                //buf[cursor - 2] = (result as u8) & 0xff;
                state.memory[(state.sp - 1) as usize] = ((result >> 8) as u8) & 0xff;
                state.memory[(state.sp - 2) as usize] = (result as u8) & 0xff;
                state.sp -= 2;
                state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                incr = false;
                //printstate = true;
            }

            //ret
            0xc9 => {
                state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                state.sp += 2;
            }

            //cz
            0xcc => {
                if state.condition.z {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //cnz
            0xc4 => {
                if !state.condition.z {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //rz
            0xc8 => {
                if state.condition.z {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //rnz
            0xc0 => {
                if !state.condition.z {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //cnc
            0xd4 => {
                if !state.condition.cy {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //cc
            0xdc => {
                if state.condition.cy {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //rnc
            0xd0 => {
                if !state.condition.cy {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //rc
            0xd8 => {
                if state.condition.cy {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //cpo
            0xe4 => {
                if state.condition.p {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //cpe
            0xec => {
                if !state.condition.p {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //rpo
            0xe0 => {
                if state.condition.p {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //rpe
            0xe8 => {
                if !state.condition.p {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //cp
            0xf4 => {
                if state.condition.s {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //cm
            0xfc => {
                if !state.condition.s {
                    let result = (state.pc as u16) + 2;
                    buf[cursor - 1] = ((result >> 8) as u8) & 0xff;
                    buf[cursor - 2] = (result as u8) & 0xff;
                    state.sp -= 2;
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                } else {
                    state.pc += 2;
                }
            }

            //rp
            0xf0 => {
                if state.condition.s {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //rm
            0xf8 => {
                if !state.condition.s {
                    state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                } else {
                    state.pc += 2;
                }
            }

            //pchl
            //return to
            0xe9 => {}

            //rst
            //return to
            0xc7 => {}

            // Everything else (unimplemented)
            _ => {
                run_emu = unimplemented(&buf[cursor]);
            }
        }
        if incr {
            state.pc += 1;
        }
        if printstate {
            println!("{:?}", state);
        }
        
    }
}

fn unimplemented(hexcode: &u8) -> bool {
    // If the instruction isn't implemented yet
    println!("Unimplemented instruction : {:02x}", hexcode);
    false
}
