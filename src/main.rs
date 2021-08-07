extern crate sdl2;

use std::fs;
use std::io::prelude::*;
use std::process;
mod condition_codes;
mod disassembler;
mod intel8080_state;
mod sounds;
use intel8080_state::StateIntel8080;
use sounds::Invaderwavs;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::time::Duration;

fn main() {
    let file_name = String::from("invaders");

    let mut buf = Vec::new();
    let mut file_in = fs::File::open(file_name).unwrap();
    file_in.read_to_end(&mut buf).unwrap();

    // If we need to print out the full disassembly
    //disassembler::print_all(&buf);

    let mut intel_8080_state: StateIntel8080 = Default::default();
    let mut sound_state: Invaderwavs = Default::default();

    // Loads all the sounds needed for the game, plays the intro sound
    sound_state.load_sounds();
    sound_state.play_sound(8);
    intel_8080_state.init_mem(&buf);

    // Main emulation function
    run_emulation(&mut intel_8080_state, &buf);

    // Print out the current state (debugging)
    //println!("{:?}", intel_8080_state);
    //print_registers(&intel_8080_state);
}

fn run_emulation(state: &mut StateIntel8080, buf: &Vec<u8>) {
    // Loop control and current instruction location
    let mut run_emu: bool = true;
    let mut cursor: usize;
    let mut incr: bool;
    let mut printstate: bool;
    let mut count = 0;
    let maxcount = 45000;

    // Utilizes example code from https://docs.rs/sdl2/0.34.5/sdl2/ and
    // SDL2 examples provided by https://github.com/Rust-SDL2/rust-sdl2 and 
    //https://nukep.github.io/rust-sdl2/sdl2/event/struct.EventPump.html
    let sdl_context = sdl2::init().expect("init failure");
    let video_subsystem = sdl_context.video().expect("video subsysteam failure");

    let window = video_subsystem
        .window("Space invaders", 250, 300)
        .position_centered()
        .build()
        .expect("video subsysteam init failure");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("canvas failure");

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut i = 0;
    
    //let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                //left
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    state.input_1 |= 0x20;
                },
                //right
                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                    state.input_1 |= 0x40
                },

                //fire
                Event::KeyDown { keycode: Some(Keycode::Period), .. } => {
                    state.input_1 |= 0x10
                },

                //insert coin
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    state.input_1 |= 0x1
                },

                //1 player
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    state.input_1 |= 0x02
                },
                //left
                Event::KeyUp { keycode: Some(Keycode::Z), .. } => {
                    state.input_1 &= !0x20
                },
                //right
                Event::KeyUp { keycode: Some(Keycode::X), .. } => {
                    state.input_1 &= !0x40
                },

                //fire
                Event::KeyUp { keycode: Some(Keycode::Period), .. } => {
                    state.input_1 &= !0x10
                },

                //insert coin
                Event::KeyUp { keycode: Some(Keycode::C), .. } => {
                    state.input_1 &= !0x1
                },

                //1 player
                Event::KeyUp { keycode: Some(Keycode::Num1), .. } => {
                    state.input_1 &= !0x02
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        while run_emu {
            incr = true;
            printstate = false;
            cursor = state.pc as usize;
            //print_registers(&state);
            //println!("");
            //print!("{:?} ", count);
            //print!("{:04x} ", cursor);
            //print!("{:02x} ", buf[cursor]);
            //disassembler::get_single(&buf, cursor);
            count += 1;
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
                // DCR D
                0x15 => {
                    state.d = state.d.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.d as u16);
                }
                // DCR C
                0x0d => {
                    state.c = state.c.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.c as u16);
                }
                // DCR A
                0x3d => {
                    state.a = state.a.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.a as u16);
                }
                // DCR E
                0x1d => {
                    state.e = state.e.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.e as u16);
                }
                // DCR H
                0x25 => {
                    state.h = state.h.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.h as u16);
                }
                // DCR L
                0x2d => {
                    state.l = state.l.wrapping_sub(1);
                    state.condition.set_dcr_flags(state.l as u16);
                }
                // MVI B,byte
                0x06 => {
                    state.b = buf[cursor + 1];
                    state.pc += 1;
                }
                // MVI D,byte
                0x16 => {
                    state.d = buf[cursor + 1];
                    state.pc += 1;
                }
                // MVI E,byte
                0x1e => {
                    state.e = buf[cursor + 1];
                    state.pc += 1;
                }
                // MVI L,byte
                0x2e => {
                    state.l = buf[cursor + 1];
                    state.pc += 1;
                }
                // DAD B
                0x09 => {
                    let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                    let bc: u16 = ((state.b as u16) << 8) | state.c as u16;
                    let result = hl.overflowing_add(bc);
                    state.h = (result.0 >> 8) as u8;
                    state.l = result.0 as u8;
                    state.condition.cy = result.1;
                }
                // DAD D
                0x19 => {
                    let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                    let de: u16 = ((state.d as u16) << 8) | state.e as u16;
                    let result = hl.overflowing_add(de);
                    state.h = (result.0 >> 8) as u8;
                    state.l = result.0 as u8;
                    state.condition.cy = result.1;
                }
                // DAD H
                0x29 => {
                    let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                    let result = hl.overflowing_add(hl);
                    state.h = (result.0 >> 8) as u8;
                    state.l = result.0 as u8;
                    state.condition.cy = result.1;
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
                    let result: u16 = (state.memory[mem_offset as usize] as u16).wrapping_add(1);
                    state.condition.set_inr_flags(result);
                    state.memory[mem_offset as usize] = result as u8;
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
                // MOV B,A
                0x47 => {
                    state.b = state.a;
                }
                // MOV C,B
                0x48 => {
                    state.c = state.b;
                }
                // MOV C,D
                0x4a => {
                    state.c = state.d;
                }
                // MOV C,E
                0x4b => {
                    state.c = state.e;
                }
                // MOV C,H
                0x4c => {
                    state.c = state.h;
                }
                // MOV D,C
                0x51 => {
                    state.d = state.c;
                }
                // MOV D,E
                0x53 => {
                    state.d = state.e;
                }
                // MOV D,H
                0x54 => {
                    state.d = state.h;
                }
                // MOV D,L
                0x55 => {
                    state.d = state.l;
                }
                // MOV E,D
                0x5a => {
                    state.e = state.d;
                }
                // MOV E,H
                0x5c => {
                    state.e = state.h;
                }
                // MOV E,L
                0x5d => {
                    state.e = state.l;
                }
                // MOV H,E
                0x63 => {
                    state.h = state.e;
                }
                // MOV H,L
                0x65 => {
                    state.h = state.l;
                }
                // MOV L,B
                0x68 => {
                    state.l = state.b;
                }
                // MOV L,C
                0x69 => {
                    state.l = state.c;
                }
                // MOV L,H
                0x6c => {
                    state.l = state.h;
                }
                // MOV A,L
                0x7d => {
                    state.a = state.l;
                }
                // MOV C,A
                0x4f => {
                    state.c = state.a;
                }
                // MOV E,C
                0x59 => {
                    state.e = state.c;
                }
                // MOV L,E
                0x6b => {
                    state.l = state.e;
                }
                // MOV B,L
                0x45 => {
                    state.b = state.l;
                }
                // MOV D,B
                0x50 => {
                    state.d = state.b;
                }
                // MOV H,B
                0x60 => {
                    state.h = state.b;
                }
                // MOV H,D
                0x62 => {
                    state.h = state.d;
                }
                // MOV H,A
                0x67 => {
                    state.h = state.a;
                }
                // MOV D,M
                0x56 => {
                    let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                    state.d = state.memory[mem_offset as usize];
                }
                // MOV D,A
                0x57 => {
                    state.d = state.a;
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
                // MOV B,H
                0x44 => {
                    state.b = state.h;
                }
                // MOV E,B
                0x58 => {
                    state.e = state.b;
                }
                // MOV E,A
                0x5f => {
                    state.e = state.a;
                }
                // MOV H,C
                0x61 => {
                    state.h = state.c;
                }
                // MOV L,D
                0x6a => {
                    state.l = state.d;
                }
                // MOV C,L
                0x4d => {
                    state.c = state.l;
                }
                // MOV A,B
                0x78 => {
                    state.a = state.b;
                }
                // MOV A,C
                0x79 => {
                    state.a = state.c;
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
                    //let x: u8 = state.a;
                    state.a = state.a.rotate_right(1);
                    //state.a = ((x & 1) << 7) | (x >> 1);
                    state.condition.cy = 1 == (state.a & 1);
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
                    let result: u16 = (state.a as u16) + (state.memory[mem_offset as usize] as u16);
                    state.condition.set_add_flags(result);
                    state.a = result as u8;
                }
                // ADD A
                0x87 => {
                    let result: u16 = (state.a as u16) + (state.a as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC B
                0x88 => {
                    let result: u16 =
                        (state.a as u16) + (state.b as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC C
                0x89 => {
                    let result: u16 =
                        (state.a as u16) + (state.c as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                //SUB B
                0x90 => {
                    let result = state.a.overflowing_sub(state.b);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB C
                0x91 => {
                    let result = state.a.overflowing_sub(state.c);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB D
                0x92 => {
                    let result = state.a.overflowing_sub(state.d);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB E
                0x93 => {
                    let result = state.a.overflowing_sub(state.e);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB H
                0x94 => {
                    let result = state.a.overflowing_sub(state.h);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB L
                0x95 => {
                    let result = state.a.overflowing_sub(state.l);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB M
                0x96 => {
                    let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                    let result = state.a.overflowing_sub(state.memory[mem_offset as usize]);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                //SUB A
                0x97 => {
                    let result = state.a.overflowing_sub(state.a);
                    state.condition.set_dcr_flags(result.0 as u16);
                    state.condition.cy = result.1;
                    state.a = result.0;
                }
                // ADC D
                0x8a => {
                    let result: u16 =
                        (state.a as u16) + (state.d as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC E
                0x8b => {
                    let result: u16 =
                        (state.a as u16) + (state.e as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC H
                0x8c => {
                    let result: u16 =
                        (state.a as u16) + (state.h as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC L
                0x8d => {
                    let result: u16 =
                        (state.a as u16) + (state.l as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADC Mem
                0x8e => {
                    let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                    let result: u16 = (state.a as u16)
                        + (state.memory[mem_offset as usize] as u16)
                        + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = result as u8;
                }
                // ADC B
                0x8f => {
                    let result: u16 =
                        (state.a as u16) + (state.a as u16) + (state.condition.cy as u16);
                    state.condition.set_add_flags(result);
                    state.a = (result as u8) & 0xff;
                }
                // ADI byte
                0xc6 => {
                    let result: u16 = (state.a as u16) + (buf[cursor + 1] as u16);
                    state.condition.set_add_flags(result);
                    state.a = result as u8;
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
                    let x = state.a.overflowing_sub(state.h);
                    state.condition.z = x.0 == 0;
                    state.condition.s = 0x80 == (x.0 & 0x80);
                    state.condition.set_parity_flag(x.0);
                    state.condition.cy = !x.1;
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

                //EI
                0xfb => {
                    state.interrupts = true;
                }

                //DI
                0xf3 => {
                    state.interrupts = false;
                }

                //in -says to leave unimplemented and return to later
                0xdb => {
                    let emu_port = buf[cursor + 1];
                    match emu_port {
                        0 => {
                            state.a = 0xf;
                        }
                        1 => {
                            state.a = state.input_1;
                        }
                        2 => {
                            state.a = state.input_2;
                        }
                        3 => {
                            let visual = ((state.shift_1 as u16) << 8) | (state.shift_0 as u16);
                            state.a = (visual >> (8 - (state.shift_offset as u16))) as u8;
                        }
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
                        }
                        3 => {
                            state.output_3 = x;
                        }
                        4 => {
                            state.shift_0 = state.shift_1;
                            state.shift_1 = x;
                        }
                        5 => {
                            state.output_5 = x;
                        }
                        6 => {}
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
                0xca => {
                    if state.condition.z {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //JC
                0xda => {
                    if state.condition.cy {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //JNC
                0xd2 => {
                    if !state.condition.cy {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //JPO
                0xe2 => {
                    if !state.condition.p {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //jpe
                0xea => {
                    if state.condition.p {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //jp (plus)
                0xf2 => {
                    print_registers(&state);
                    if !state.condition.s {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                //jm (minus)
                0xfa => {
                    if state.condition.s {
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }
                // SUI D8
                0xd6 => {
                    let result = state.a.overflowing_sub(buf[cursor + 1]);
                    state.condition.set_sub_flags(result.0 as u16);
                    state.condition.cy = result.1;

                    state.pc += 1;

                    state.a = result.0;
                }
                // SBI D8
                0xde => {
                    let result = state
                        .a
                        .overflowing_sub(buf[cursor + 1] + (state.condition.cy as u8));
                    state.condition.set_sub_flags(result.0 as u16);
                    state.condition.cy = result.1;

                    state.pc += 1;

                    state.a = result.0;
                }
                // ORI D8
                0xf6 => {
                    let result = state.a | buf[cursor + 1];
                    state.condition.set_dcr_flags(result as u16);
                    state.condition.cy = false;
                    state.a = result;
                    state.pc += 1;
                }

                // XRI D8
                0xee => {
                    let result = state.a ^ buf[cursor + 1];
                    state.condition.set_dcr_flags(result as u16);
                    state.condition.cy = false;
                    state.a = result;
                    state.pc += 1;
                }

                // CC ADR
                0xdc => {
                    if state.condition.cy {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CNC ADR
                0xd4 => {
                    if !state.condition.cy {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CPO ADR
                0xe4 => {
                    if !state.condition.p {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((state.memory[cursor + 2] as u16) << 8)
                            | (state.memory[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CM ADR
                0xfc => {
                    if state.condition.s {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CNZ ADR
                0xc4 => {
                    if !state.condition.z {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CPE ADR
                0xec => {
                    if state.condition.p {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CP ADR
                0xf4 => {
                    if !state.condition.s {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
                    } else {
                        state.pc += 2;
                    }
                }

                // CZ ADR
                0xcc => {
                    if state.condition.z {
                        let result = (state.pc as u16) + 3;
                        state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                        state.memory[(state.sp - 2) as usize] = result as u8;
                        state.sp -= 2;
                        state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                        incr = false;
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
                    state.pc = state.memory[state.sp as usize] as u16
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    //state.pc = (buf[cursor] as u16) | ((buf[cursor + 1] as u16) << 8);
                    state.sp += 2;
                    incr = false;
                }

                //rz
                0xc8 => {
                    if state.condition.z {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rnz
                0xc0 => {
                    if !state.condition.z {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rnc
                0xd0 => {
                    if !state.condition.cy {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rc
                0xd8 => {
                    if state.condition.cy {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rpo
                0xe0 => {
                    if !state.condition.p {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rpe
                0xe8 => {
                    if state.condition.p {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rp
                0xf0 => {
                    if !state.condition.s {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
                    }
                }

                //rm
                0xf8 => {
                    if state.condition.s {
                        state.pc = (state.memory[state.sp as usize] as u16)
                            | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                        state.sp += 2;
                        incr = false;
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

                    if state.condition.z {
                        z_int = 1;
                    }
                    if state.condition.s {
                        s_int = 1;
                    }
                    if state.condition.p {
                        p_int = 1;
                    }
                    if state.condition.cy {
                        cy_int = 1;
                    }
                    if state.condition.ac {
                        ac_int = 1;
                    }

                    let psw: u8 = z_int | s_int << 1 | p_int << 2 | cy_int << 3 | ac_int << 4;

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
        draw_screen(&mut canvas, &state);
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn unimplemented(hexcode: &u8) -> bool {
    // If the instruction isn't implemented yet
    println!("Unimplemented instruction : {:02x}", hexcode);
    false
}

#[allow(dead_code)]
fn print_registers(state: &StateIntel8080) {
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
    print!("i = {:?}, ", state.interrupts);
    println!("{:?}", state.condition);
}

fn get_bits(vram_byte: u8, bit_vector: &mut Vec<bool>) {
    bit_vector[0] = (vram_byte & 0b1) != 0;
    bit_vector[1] = (vram_byte & 0b10) != 0;
    bit_vector[2] = (vram_byte & 0b100) != 0;
    bit_vector[3] = (vram_byte & 0b1000) != 0;
    bit_vector[4] = (vram_byte & 0b10000) != 0;
    bit_vector[5] = (vram_byte & 0b100000) != 0;
    bit_vector[6] = (vram_byte & 0b1000000) != 0;
    bit_vector[7] = (vram_byte & 0b10000000) != 0;
}

// Utilizes example code from https://docs.rs/sdl2/0.34.5/sdl2/ and
// SDL2 examples provided by https://github.com/Rust-SDL2/rust-sdl2
fn draw_screen(canvas: &mut WindowCanvas, state: &StateIntel8080) {
    let texture_creator = canvas.texture_creator();

    let start_vram_index: usize = 0x2400;
    let end_vram_index: usize = 0x4000;

    let vram = &state.memory[start_vram_index..end_vram_index];
    let mut bit_vector = vec![false; 8];

    let mut x: usize = 0;
    let mut y: usize = 0;

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB332, 256, 224)
        .unwrap();

    texture
        .with_lock(None, |buf: &mut [u8], pitch: usize| {
            for byte in vram {
                get_bits(*byte, &mut bit_vector);
                for bit in &bit_vector {
                    if *bit {
                        buf[x] = 255;
                    } else {
                        buf[x] = 0;
                    }
                    x += 1;
                }
            }
        })
        .unwrap();

    //println!("{:?}", vram);

    //canvas.copy(&texture, None, None).unwrap();

    canvas
        .copy_ex(
            &texture,
            None,
            Rect::new(13, 0, 224, 256),
            270 as f64,
            None,
            false,
            false,
        )
        .unwrap();

    //let mut points = Vec::new();
    //for byte in vram {
    //    get_bits(*byte, &mut bit_vector);
    //
    //        for bit in &bit_vector {
    //           x += 1;
    //           if *bit {
    //               points.push(Point::new(x, y));
    //          }
    //       }
    //
    //       if x == 224 {
    //           y += 1;
    //           x = 0;
    //       }
    //  }

    //print!("{:?}", points);

    //canvas.set_draw_color(Color::RGB(255, 255, 255));
    //canvas.draw_points(&points[..]);
    canvas.present();
}
