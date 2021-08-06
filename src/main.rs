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

    intel_8080_state.init_mem(& buf);

    // Main emulation function
    run_emulation(&mut intel_8080_state, & buf);

    // Print out the current state (debugging)
    //println!("{:?}", intel_8080_state);
    //print_registers(&intel_8080_state);
}

fn run_emulation(state: &mut StateIntel8080, buf: & Vec<u8>) {
    // Loop control and current instruction location
    let mut run_emu: bool = true;
    let mut cursor: usize;
    let mut incr: bool = true;
    let mut printstate: bool = false;
    let mut count = 0;
    let maxcount = 1547;

    while run_emu {
        incr = true;
        printstate = false;
        cursor = state.pc as usize;
        print_registers(&state);
        print!("{:?} ", count);
        print!("{:04x} ", cursor);
        print!("{:02x} ", buf[cursor]);
        disassembler::get_single(&buf, cursor);
        count+= 1;
        match buf[cursor] {
            // NOP
            0x00 => {}
            // LXI B,word
            0x01 => {
                state.b = buf[cursor + 2];
                state.c = buf[cursor + 1];
                state.pc += 2;
            }
            // INR B
            0x04 => {
                state.b = state.b.wrapping_add(1);
                state.condition.set_inr_flags(state.b as u16);
            }
            // DCR B
            0x05 => {
                state.b = state.b.wrapping_sub(1);
                state.condition.set_dcr_flags(state.b as u16);
            }
            // DCR C
            0x0d => {
                state.d = state.d.wrapping_sub(1);
                state.condition.set_dcr_flags(state.d as u16);
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
                state.d = buf[cursor + 2];
                state.e = buf[cursor + 1];
                state.pc += 2;
            }
            // LXI H,word
            0x21 => {
                state.h = buf[cursor + 2];
                state.l = buf[cursor + 1];
                state.pc += 2;
            }
            // LXI SP,word
            0x31 => {
                state.sp = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.pc += 2;
            }
            // INX D
            0x13 => {
                state.e = state.e.wrapping_add(1);
                if state.e == 0 {
                    state.d = state.d.wrapping_add(1);
                }
            }
            // INX H
            0x23 => {
                state.l = state.l.wrapping_add(1);
                if state.l == 0 {
                    state.h = state.h.wrapping_add(1);
                }
            }
            // LDAX D
            0x1a => {
                let mem_offset: u16 = (state.d as u16) << 8 | state.e as u16;
                state.a = state.memory[mem_offset as usize];
            }
            // MOV M,A
            0x77 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.a;
            }
            // STA word
            0x32 => {
                let mem_offset: u16 = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.memory[mem_offset as usize] = state.a;
                state.pc += 2;
            }
            // LDA word
            0x3a => {
                let mem_offset: u16 = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.a = state.memory[mem_offset as usize]; 
                state.pc += 2;
            }
            // INR C
            0x0c => {
                state.c = state.c.wrapping_add(1);
                state.condition.set_inr_flags(state.c as u16);
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
            // MVI M,byte
            0x36 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = buf[cursor + 1];
                state.pc += 1;
            }
            // INR D
            0x14 => {
                state.d = state.d.wrapping_add(1);
                state.condition.set_inr_flags(state.d as u16);
            }
            // INR E
            0x1c => {
                state.e = state.e.wrapping_add(1);
                state.condition.set_inr_flags(state.e as u16);
            }
            // INR H
            0x24 => {
                state.h = state.h.wrapping_add(1);
                state.condition.set_inr_flags(state.h as u16);
            }
            // INR L
            0x2c => {
                state.l = state.l.wrapping_add(1);
                state.condition.set_inr_flags(state.l as u16);
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
                state.a = state.a.wrapping_add(1);
                state.condition.set_inr_flags(state.a as u16);
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
            // MOV D,M
            0x56 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.d = state.memory[mem_offset as usize];
            }
            // MOV E,M
            0x5e => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.e = state.memory[mem_offset as usize];
            }
            // MOV H,M
            0x66 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.h = state.memory[mem_offset as usize];
            }
            // MOV L,A
            0x6f => {
                state.l = state.a;
            }
            // MOV A,D
            0x7a => {
                state.a = state.d;
            }
            // MOV A,E
            0x7b => {
                state.a = state.e;
            }
            // MOV A,H
            0x7c => {
                state.a = state.h;
            }
            // MOV A,M
            0x7e => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.a = state.memory[mem_offset as usize];
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
            // ANA A
            0xa7 => {
                let x: u8 = state.a & state.a;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
                state.pc += 1;
            }
            // XRA A
            0xaf => {
                let x: u8 = state.a ^ state.a;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
                state.pc += 1;
            }
            // ANI
            0xe6 => {
                let x: u8 = state.a & buf[cursor + 1];
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                //Data book says ANI clears CY
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
                state.pc += 1;
            }
            // XCHG
            0xeb => {
                let save1: u8 = state.d;
                let save2: u8 = state.e;
                state.d = state.h;
                state.e = state.l;
                state.h = save1;
                state.l = save2;
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
                //let x: u8 = state.a - buf[cursor + 1];

                let x = state.a.overflowing_sub(buf[cursor + 1]);
                state.condition.z = x.0 == 0;
                state.condition.s = 0x80 == (x.0 & 0x80);
                state.condition.set_parity_flag(x.0);
                //state.condition.cy = state.a < buf[cursor + 1];
                state.condition.cy = x.1;
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
            0xdb => {
                let emu_port = buf[cursor + 1];
                match emu_port {
                    0 => {
                        state.a = 0xf;
                    },
                    1 => {
                        state.a = state.input_1;
                    },                    
                    2 => {
                        state.a = state.input_2;
                    },
                    3 => {
                        let visual = ((state.shift_1 as u16) << 8) | (state.shift_0 as u16);
                        state.a = (visual >> (8 - (state.shift_offset as u16))) as u8;
                    },
                    _ => {
                        state.a = 0;
                    }
                }
                state.pc += 1;
            }

            // OUT
            0xd3 => {
                let emu_port = buf[cursor + 1];
                let x: u8 = state.a;
                match emu_port {
                    2 => {
                        state.shift_offset = x & 0x7;
                    },
                    3 => {
                        state.output_3 = x;
                    },                    
                    4 => {
                        state.shift_0 = state.shift_1;
                        state.shift_1 = x;
                    },
                    5 => {
                        state.output_5 = x;
                    },
                    6 => {},
                    _ => {
                        run_emu = unimplemented(&buf[cursor]);
                    }
                }
                state.pc += 1;
            }

            //jmp
            0xc3 => {
                state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                incr = false;
            }

            //JNZ
            0xc2 => {
                if !state.condition.z {
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
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
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp -= 2;
                state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                incr = false;
            }

            //ret
            0xc9 => {
                state.pc = state.memory[state.sp as usize] as u16 | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                //state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                state.sp += 2;
                incr = false;
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
            // POP B
            0xc1 => {
                state.c = state.memory[state.sp as usize];
                state.b = state.memory[state.sp as usize + 1];
                state.sp += 2;
            }
            // PUSH B
            0xc5 => {
                state.memory[state.sp as usize - 1] = state.b;
                state.memory[state.sp as usize - 2] = state.c;
                state.sp = state.sp - 2;
            }
            // POP D
            0xd1 => {
                state.e = state.memory[state.sp as usize];
                state.d = state.memory[state.sp as usize + 1];
                state.sp += 2;
            }
            // PUSH D
            0xd5 => {
                state.memory[state.sp as usize - 1] = state.d;
                state.memory[state.sp as usize - 2] = state.e;
                state.sp = state.sp - 2;
            }
            // POP H
            0xe1 => {
                state.l = state.memory[state.sp as usize];
                state.h = state.memory[state.sp as usize + 1];
                state.sp += 2;
            }
            // PUSH H
            0xe5 => {
                state.memory[state.sp as usize - 1] = state.h;
                state.memory[state.sp as usize - 2] = state.l;
                state.sp = state.sp - 2;
            }
            // POP PSW
            0xf1 => {
                state.a = state.memory[state.sp as usize + 1];
                let psw: u8 = state.memory[state.sp as usize];
                state.condition.z = 0x01 == (psw & 0x01);
                state.condition.s = 0x02 == (psw & 0x02);
                state.condition.p = 0x04 == (psw & 0x04);
                state.condition.cy = 0x05 == (psw & 0x05);
                state.condition.ac = 0x10 == (psw & 0x10);
                state.sp += 2;
            }
            // PUSH PSW
            0xf5 => {
                state.memory[state.sp as usize - 1] = state.a;
                let mut z_int: u8 = 0;
                let mut s_int: u8 = 0;
                let mut p_int: u8 = 0;
                let mut cy_int: u8 = 0;
                let mut ac_int: u8 = 0;

                if state.condition.z {z_int = 1;}
                if state.condition.s {s_int = 1;}
                if state.condition.p {p_int = 1;}
                if state.condition.cy {cy_int = 1;}
                if state.condition.ac {ac_int = 1;}

                let psw: u8 = z_int | 
                    s_int << 1 | 
                    p_int << 2 | 
                    cy_int << 3 | 
                    ac_int << 4;

                state.memory[state.sp as usize - 2] = psw;
                state.sp = state.sp - 2;
            }
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
        if count > maxcount {
            break;
        }
    }
}

fn unimplemented(hexcode: &u8) -> bool {
    // If the instruction isn't implemented yet
    println!("Unimplemented instruction : {:02x}", hexcode);
    false
}

#[allow(dead_code)]
fn print_registers(state: & StateIntel8080)
{
    //print!("b = {:02x}, ", state.b);
    //print!("c = {:02x}, ", state.c);
    //print!("d = {:02x}, ", state.d);
    //print!("e = {:02x}, ", state.e);
    //print!("h = {:02x}, ", state.h);
    //print!("l = {:02x}, ", state.l);
    print!("a = {:02x}, ", state.a);
    print!("bc = {:02x}{:02x}, ", state.b, state.c);
    print!("de = {:02x}{:02x}, ", state.d, state.e);
    print!("hl = {:02x}{:02x}, ", state.h, state.l);
    print!("pc = {:04x}, ", state.pc);
    print!("sp = {:04x}, ", state.sp);
    println!("{:?}", state.condition);
}
