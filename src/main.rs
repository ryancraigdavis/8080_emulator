extern crate sdl2;

use std::fs;
use std::io::prelude::*;
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
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time;

// Main loop - Initializes video and kicks off emulation
fn main() {
    // Read in (combined) invaders file
    let file_name = String::from("invaders");

    // Load file into vector
    let mut buf = Vec::new();
    let mut file_in = fs::File::open(file_name).expect("file failure");
    file_in.read_to_end(&mut buf).unwrap();

    // Initialize intel 8080 state
    let mut intel_8080_state: StateIntel8080 = Default::default();

    // Initialize sound
    let mut sound_state: Invaderwavs = Default::default();

    // Loads all the sounds needed for the game, plays the intro sound
    sound_state.load_sounds();

    // Load main memory
    intel_8080_state.init_mem(&buf);

    // Utilizes example code from https://docs.rs/sdl2/0.34.5/sdl2/ and
    // code from SDL2 examples provided by https://github.com/Rust-SDL2/rust-sdl2
    let sdl_context = sdl2::init().expect("init failure");
    let video_subsystem = sdl_context.video().expect("video subsysteam failure");

    let window = video_subsystem
        .window("Space invaders", 250, 300)
        .position_centered()
        .build()
        .expect("video subsysteam init failure");

    let mut canvas = window.into_canvas().build().expect("canvas failure");

    // Used to clear screen, from SDL2 examples
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // The intel 8080 generates an interrupt after half the screen is rendered
    let mut top: bool;

    // SDL2 loop, from examples
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                // Key mappings
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // Key down
                // Left
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => {
                    intel_8080_state.input_1 |= 0x20;
                }
                // Right
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => intel_8080_state.input_1 |= 0x40,

                // Fire
                Event::KeyDown {
                    keycode: Some(Keycode::Period),
                    ..
                } => intel_8080_state.input_1 |= 0x10,

                // Insert coin
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => intel_8080_state.input_1 |= 0x1,

                // 1 player
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => intel_8080_state.input_1 |= 0x04,

                // Key up
                // Left
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => intel_8080_state.input_1 &= !0x20,
                // Right
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => intel_8080_state.input_1 &= !0x40,

                // Fire
                Event::KeyUp {
                    keycode: Some(Keycode::Period),
                    ..
                } => intel_8080_state.input_1 &= !0x10,

                // Insert coin
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => intel_8080_state.input_1 &= !0x1,

                // 1 player
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => intel_8080_state.input_1 &= !0x04,

                _ => {}
            }
        }

        // Clear screen every loop
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Start with top half, run emulation
        top = true;
        run_emulation(&mut intel_8080_state, &buf, &mut sound_state);

        // If interrupts, then draw the screen (top half)
        if intel_8080_state.interrupts {
            intel_8080_state.generate_interrupt(1);
            draw_screen(&mut canvas, &intel_8080_state, top);
        }

        // Move onto bottom half
        top = false;
        run_emulation(&mut intel_8080_state, &buf, &mut sound_state);

        // If interrupts, then draw the bottom half
        if intel_8080_state.interrupts {
            intel_8080_state.generate_interrupt(2);
            draw_screen(&mut canvas, &intel_8080_state, top);
        }

        // Sleep for 1/60 of a second, for 60hz output
        ::std::thread::sleep(time::Duration::from_micros(16667));
    }

    print!("Executed finished");
}

// Emulation loop, handles intel 8080 instructions
fn run_emulation(state: &mut StateIntel8080, buf: &Vec<u8>, sound_state: &mut sound_state) {
    // Loop control and current instruction location
    let mut incr: bool;

    // The intel 8080 runs at 2 megahertz
    let clock_rate = 2_000_000;
    // We want it to execuate at 60hz refresh rate
    let cycles_per_frame = clock_rate / 60;
    let mut cycle_count: u32 = 0;

    let mut cursor: usize;
    let cycle_offset = 6;

    // Debug
    //print_registers(&state);
    //println!("");

    // Run the loop for a single frame's time, 1/60th of a second
    // Divide by two because we're rendering half the frame
    while cycle_count < (cycles_per_frame / 2) {
        incr = true;
        cursor = state.pc as usize;

        // Debug
        //print!("{:04x} ", cursor);
        //print!("{:02x} ", buf[cursor]);
        //disassembler::get_single(&buf, cursor);

        // The buffer is read-only, use this for safer execution
        // Heavy use of https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf
        // and Emulator 101 guide for instruction implementation
        match buf[cursor] {
            // NOP
            0x00 => {}
            // LXI B,word
            0x01 => {
                state.b = buf[cursor + 2];
                state.c = buf[cursor + 1];
                state.pc += 2;
            }
            // STAX B
            0x02 => {
                let mem_offset: u16 = ((state.b as u16) << 8) | (state.c as u16);
                state.memory[mem_offset as usize] = state.a;
            }
            // INX B
            0x03 => {
                state.c = state.c.wrapping_add(1);
                if state.c == 0 {
                    state.b = state.b.wrapping_add(1);
                }
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
            // MVI B,byte
            0x06 => {
                state.b = buf[cursor + 1];
                state.pc += 1;
            }
            // RLC
            0x07 => {
                state.condition.cy = (state.a & 0b10000000) != 0;
                state.a = state.a.rotate_left(1);
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
            // DAD SP
            0x39 => {
                let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                let result = hl.overflowing_add(state.sp);
                state.h = (result.0 >> 8) as u8;
                state.l = result.0 as u8;
                state.condition.cy = result.1;
            }
            // PCHL
            0xe9 => {
                let hl: u16 = ((state.h as u16) << 8) | state.l as u16;
                state.pc = hl;
                incr = false;
            }
            // RAL
            0x17 => {
                let init_carry = state.condition.cy;
                state.condition.cy = (state.a & 0b10000000) != 0;
                state.a = state.a.rotate_left(1);
                if init_carry {
                    state.a = state.a | 1;
                } else {
                    state.a = state.a & 0b11111110;
                }
            }
            // RAR
            0x1f => {
                let init_carry = state.condition.cy;
                state.condition.cy = (state.a & 1) != 0;
                state.a = state.a.rotate_right(1);
                if init_carry {
                    state.a = state.a | 0b10000000;
                } else {
                    state.a = state.a & 0b01111111;
                }
            }
            // LDAX B
            0x0a => {
                let mem_offset: u16 = (state.b as u16) << 8 | state.c as u16;
                state.a = state.memory[mem_offset as usize];
            }
            // LDAX D
            0x20 => {
                let mem_offset: u16 = (state.d as u16) << 8 | state.e as u16;
                state.a = state.memory[mem_offset as usize];
            }
            // INR C
            0x0c => {
                state.c = state.c.wrapping_add(1);
                state.condition.set_inr_flags(state.c as u16);
            }
            // DCR C
            0x0d => {
                state.c = state.c.wrapping_sub(1);
                state.condition.set_dcr_flags(state.c as u16);
            }
            // MVI C,byte
            0x0e => {
                state.c = buf[cursor + 1];
                state.pc += 1;
            }
            // STAX D
            0x12 => {
                let mem_offset: u16 = ((state.d as u16) << 8) | (state.e as u16);
                state.memory[mem_offset as usize] = state.a;
            }
            // DCR D
            0x15 => {
                state.d = state.d.wrapping_sub(1);
                state.condition.set_dcr_flags(state.d as u16);
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
            // SHLD ADR
            0x22 => {
                let mem_offset = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.memory[mem_offset as usize] = state.l;
                state.memory[(mem_offset + 1) as usize] = state.h;

                state.pc += 2;
            }
            // LHLD ADR
            0x2a => {
                let new_addr = (buf[cursor + 2] as u16) << 8 | buf[cursor + 1] as u16;
                state.l = state.memory[new_addr as usize];
                state.h = state.memory[(new_addr + 1) as usize];
                state.pc += 2;
            }
            // INX H
            0x23 => {
                state.l = state.l.wrapping_add(1);
                if state.l == 0 {
                    state.h = state.h.wrapping_add(1);
                }
            }
            // DCX H
            0x2b => {
                if state.l == 0 {
                    state.h = state.h.wrapping_sub(1);
                }
                state.l = state.l.wrapping_sub(1);
            }
            // DCX B
            0x0b => {
                if state.c == 0 {
                    state.b = state.b.wrapping_sub(1);
                }
                state.c = state.c.wrapping_sub(1);
            }
            // DCX D
            0x1b => {
                if state.e == 0 {
                    state.d = state.d.wrapping_sub(1);
                }
                state.e = state.e.wrapping_sub(1);
            }
            // INX SP
            0x33 => {
                state.sp = state.sp.wrapping_add(1);
            }
            // DCX SP
            0x3b => {
                state.sp = state.sp.wrapping_sub(1);
            }
            // SPHL
            0xf9 => {
                let hl: u16 = (state.h as u16) << 8 | state.l as u16;
                state.sp = hl;
            }
            // LDAX D
            0x1a => {
                let mem_offset: u16 = (state.d as u16) << 8 | state.e as u16;
                state.a = state.memory[mem_offset as usize];
            }
            // MOV M,B
            0x70 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.b;
            }
            // MOV M,C
            0x71 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.c;
            }
            // MOV M,D
            0x72 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.d;
            }
            // MOV M,E
            0x73 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.e;
            }
            // MOV M,H
            0x74 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.h;
            }
            // MOV M,L
            0x75 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.memory[mem_offset as usize] = state.l;
            }
            // HLT
            0x76 => {
                break;
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
            // DCR Mem
            0x35 => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                let result: u16 = (state.memory[mem_offset as usize] as u16).wrapping_sub(1);
                state.condition.set_dcr_flags(result);
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
            // MOV C,C
            0x49 => {
                state.c = state.c;
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
            // MOV D,D
            0x52 => {
                state.d = state.d;
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
            // MOV B,M
            0x46 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.b = state.memory[mem_offset as usize];
            }
            // MOV C,M
            0x4e => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.c = state.memory[mem_offset as usize];
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
            // MOV L,M
            0x6e => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                state.l = state.memory[mem_offset as usize];
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
            // DAA
            // Not validated/finished
            0x27 => {
                let four_bits_low = state.a & 0x0f;
                //let mut set_flags = false;

                //if four_bits_low > 0b1001 || state.condition.ac {
                //    state.a = state.a.wrapping_add(0x06);
                //    state.condition.ac = (four_bits_low + 0x06) > 0x10;
                //    set_flags = true;
                //} else {
                //    state.condition.ac = false;
                //}

                if four_bits_low > 0b1001 {
                    state.a = state.a.wrapping_add(0x06);
                }

                //if (state.a & 0xf0) > 0b10010000 || state.condition.cy {
                //    let result = state.a.overflowing_add(0x60);
                //   state.a = result.0;
                //  set_flags = true;

                //} else {
                //    state.condition.cy = false;
                //}

                if (state.a & 0xf0) > 0b10010000 || state.condition.cy {
                    let result = state.a.overflowing_add(0x60);
                    state.a = result.0;
                    //set_flags = true;
                    state.condition.cy = true;
                }

                state.condition.set_zero_flag(state.a as u16);
                state.condition.set_sign_flag(state.a as u16);
                state.condition.set_parity_flag(state.a);
                state.condition.set_ac_flag(state.a as u16);
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
            // ANA B
            0xa0 => {
                let x: u8 = state.a & state.b;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA C
            0xa1 => {
                let x: u8 = state.a & state.c;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA D
            0xa2 => {
                let x: u8 = state.a & state.d;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA E
            0xa3 => {
                let x: u8 = state.a & state.e;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA H
            0xa4 => {
                let x: u8 = state.a & state.h;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA L
            0xa5 => {
                let x: u8 = state.a & state.l;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            //  ANA M
            0xa6 => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                let x: u8 = state.a & state.memory[mem_offset as usize];
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // ANA A
            0xa7 => {
                let x: u8 = state.a & state.a;
                state.condition.set_zero_flag(x as u16);
                //state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA B
            0xa8 => {
                let x: u8 = state.a ^ state.b;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA C
            0xa9 => {
                let x: u8 = state.a ^ state.c;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA D
            0xaa => {
                let x: u8 = state.a ^ state.d;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA E
            0xab => {
                let x: u8 = state.a ^ state.e;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA H
            0xac => {
                let x: u8 = state.a ^ state.h;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA L
            0xad => {
                let x: u8 = state.a ^ state.l;
                state.condition.z = x == 0;
                state.condition.s = 0x80 == (x & 0x80);
                state.condition.set_parity_flag(x);
                state.condition.cy = false;
                state.condition.ac = false;
                state.a = x;
            }
            // XRA M
            0xae => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                let x: u8 = state.a ^ state.memory[mem_offset as usize];
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
            // XTHL
            0xe3 => {
                let temp1 = state.memory[state.sp as usize];
                let temp2 = state.memory[(state.sp + 1) as usize];

                state.memory[state.sp as usize] = state.l;
                state.memory[(state.sp + 1) as usize] = state.h;

                state.l = temp1;
                state.h = temp2;
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
                state.condition.cy = 1 == (state.a & 1);
                state.a = state.a.rotate_right(1);
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
            // SBB B
            0x98 => {
                let result = state
                    .a
                    .overflowing_sub(state.b.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB C
            0x99 => {
                let result = state
                    .a
                    .overflowing_sub(state.c.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB D
            0x9a => {
                let result = state
                    .a
                    .overflowing_sub(state.d.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB E
            0x9b => {
                let result = state
                    .a
                    .overflowing_sub(state.e.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB H
            0x9c => {
                let result = state
                    .a
                    .overflowing_sub(state.h.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB L
            0x9d => {
                let result = state
                    .a
                    .overflowing_sub(state.l.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB M
            0x9e => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                let result = state.a.overflowing_sub(
                    state.memory[mem_offset as usize].wrapping_add(state.condition.cy as u8),
                );
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
            }
            // SBB A
            0x9f => {
                let result = state
                    .a
                    .overflowing_sub(state.a.wrapping_add(state.condition.cy as u8));
                state.condition.set_dcr_flags(result.0 as u16);
                state.condition.cy = result.1;
                state.a = result.0;
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
                let result: u16 = (state.a as u16)
                    + (state.memory[mem_offset as usize] as u16)
                    + (state.condition.cy as u16);
                state.condition.set_add_flags(result);
                state.a = result as u8;
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
                state.condition.set_ac_flag(x.0 as u16);
                state.pc += 1;
            }
            // CMP B
            0xb8 => {
                let x = state.a.overflowing_sub(state.b);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP C
            0xb9 => {
                let x = state.a.overflowing_sub(state.c);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP D
            0xba => {
                let x = state.a.overflowing_sub(state.d);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP E
            0xbb => {
                let x = state.a.overflowing_sub(state.e);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP H
            0xbc => {
                let x = state.a.overflowing_sub(state.h);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP L
            0xbd => {
                let x = state.a.overflowing_sub(state.l);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP M
            0xbe => {
                let mem_offset: u16 = (state.h as u16) << 8 | state.l as u16;
                let x = state.a.overflowing_sub(state.memory[mem_offset as usize]);
                state.condition.set_dcr_flags(x.0 as u16);
                state.condition.cy = x.1;
            }
            // CMP A
            0xbf => {
                let x: u8 = state.a - state.a;
                state.condition.set_dcr_flags(x as u16);
                state.condition.cy = state.a < state.a;
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
                state.pc = state.pc.wrapping_add(1);
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
                        sound_state.play_sounds(x, 1);        
                    }
                    4 => {
                        state.shift_0 = state.shift_1;
                        state.shift_1 = x;
                    }
                    5 => {
                        sound_state.play_sounds(x, 2);        
                    }
                    6 => {}
                    _ => {
                        let run_emu = unimplemented(&buf[cursor]);
                        if !run_emu {
                            break;
                        }
                    }
                }
                state.pc = state.pc.wrapping_add(1);
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
            // ORA B
            0xb0 => {
                let result = state.a | state.b;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA C
            0xb1 => {
                let result = state.a | state.c;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA D
            0xb2 => {
                let result = state.a | state.d;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA E
            0xb3 => {
                let result = state.a | state.e;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA H
            0xb4 => {
                let result = state.a | state.h;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA L
            0xb5 => {
                let result = state.a | state.l;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA M
            0xb6 => {
                let mem_offset: u16 = ((state.h as u16) << 8) | (state.l as u16);
                let result = state.a | state.memory[mem_offset as usize];
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
            }
            // ORA A
            0xb7 => {
                let result = state.a | state.a;
                state.condition.set_dcr_flags(result as u16);
                state.condition.cy = false;
                state.a = result;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((state.memory[cursor + 2] as u16) << 8)
                        | (state.memory[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
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
                    state.sp = state.sp.wrapping_sub(2);
                    state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                    incr = false;
                    cycle_count += cycle_offset;
                } else {
                    state.pc += 2;
                }
            }
            // Call
            0xcd => {
                let result = (state.pc as u16).wrapping_add(3);
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = ((buf[cursor + 2] as u16) << 8) | (buf[cursor + 1] as u16);
                incr = false;
            }
            // RET
            0xc9 => {
                state.pc = state.memory[state.sp as usize] as u16
                    | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                state.sp += 2;
                incr = false;
            }
            // RZ
            0xc8 => {
                if state.condition.z {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RNZ
            0xc0 => {
                if !state.condition.z {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RNC
            0xd0 => {
                if !state.condition.cy {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RC
            0xd8 => {
                if state.condition.cy {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RPO
            0xe0 => {
                if !state.condition.p {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RPE
            0xe8 => {
                if state.condition.p {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RP
            0xf0 => {
                if !state.condition.s {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
                }
            }
            // RM
            0xf8 => {
                if state.condition.s {
                    state.pc = (state.memory[state.sp as usize] as u16)
                        | (state.memory[(state.sp + 1) as usize] as u16) << 8;
                    state.sp += 2;
                    incr = false;
                    cycle_count += cycle_offset;
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
                state.condition.cy = 0x08 == (psw & 0x08);
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
            // RST 0
            0xc7 => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0000;
                incr = false;
            }
            // RST 1
            0xcf => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0008;
                incr = false;
            }
            // RST 2
            0xd7 => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0010;
                incr = false;
            }
            // RST 3
            0xdf => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0018;
                incr = false;
            }
            // RST 4
            0xe7 => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0020;
                incr = false;
            }
            // RST 5
            0xef => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0028;
                incr = false;
            }
            // RST 6
            0xf7 => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0030;
                incr = false;
            }
            // RST 7
            0xff => {
                let result = (state.pc as u16) + 3;
                state.memory[(state.sp - 1) as usize] = (result >> 8) as u8;
                state.memory[(state.sp - 2) as usize] = result as u8;
                state.sp = state.sp.wrapping_sub(2);
                state.pc = 0x0038;
                incr = false;
            }

            // Everything else (unimplemented)
            _ => {
                let cont_run = unimplemented(&buf[cursor]);
                if !cont_run {
                    break;
                }
            }
        }

        // Update the cycle count with the current cycles taken
        cycle_count += get_cycles(buf[cursor]) as u32;

        // Increment pc unless we updated it manually
        if incr {
            state.pc = state.pc.wrapping_add(1);
        }
    }
}

// If we haven't implemented a code, stop execution
fn unimplemented(hexcode: &u8) -> bool {
    // If the instruction isn't implemented yet
    println!("Unimplemented instruction : {:02x}", hexcode);
    false
}

// Debug printing
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

// Converting bytes to bits
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

// Draws to the screen
// Utilizes example code from https://docs.rs/sdl2/0.34.5/sdl2/ and
// SDL2 examples provided by https://github.com/Rust-SDL2/rust-sdl2
fn draw_screen(canvas: &mut WindowCanvas, state: &StateIntel8080, _top: bool) {
    canvas.clear();
    let texture_creator = canvas.texture_creator();

    let start_vram_index: usize = 0x2400;
    let end_vram_index: usize = 0x4000;

    let _v_ram_offset = (end_vram_index - start_vram_index) / 2;

    //if top
    //{
    //    end_vram_index -= vram_offset;
    //}
    //else {
    //    start_vram_index += vram_offset;
    //}

    let pixel_offset: usize = 0;

    //if !top {
    //    pixel_offset = (224 * 256) / 2;
    //}

    let vram = &state.memory[start_vram_index..end_vram_index];
    let mut bit_vector = vec![false; 8];

    let mut x: usize = 0;

    // RGB 332 - for 8 bit color
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB332, 256, 224)
        .unwrap();

    texture
        .with_lock(None, |buf: &mut [u8], _pitch: usize| {
            for byte in vram {
                get_bits(*byte, &mut bit_vector);
                for bit in &bit_vector {
                    if *bit {
                        // Colors Based on visual approximations
                        // from real gameplay - https://www.youtube.com/watch?v=MU4psw3ccUI
                        if (x % 256) > 15 && (x % 256) < 80 {
                            // 8 bit color is 0brrrgggbb
                            buf[x + pixel_offset] = 0b00011100;
                        } else if (x % 256) > 200 && (x % 256) < 222 {
                            buf[x + pixel_offset] = 0b11100000;
                        } else {
                            buf[x + pixel_offset] = 255;
                        }
                    } else {
                        buf[x + pixel_offset] = 0;
                    }
                    x += 1;
                }
            }
        })
        .unwrap();
    // Use rotated rendering due to space invaders design
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

    // Display on screen
    canvas.present();
}

// Get cycles for every instruction
fn get_cycles(opcode: u8) -> u8 {
    // Extracted from https://pastraiser.com/cpu/i8080/i8080_opcodes.html using excel + vs code
    let cycles = [
        4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5,
        5, 7, 4, 4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 13, 5, 10, 10, 10, 4,
        4, 10, 13, 5, 5, 5, 7, 4, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5,
        7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, 7, 7, 7, 7,
        7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 7, 5, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4,
        4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
        4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, 11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10,
        10, 17, 17, 7, 11, 11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 17, 17, 7, 11, 11, 10,
        10, 18, 17, 11, 7, 11, 11, 5, 10, 5, 17, 17, 7, 11, 11, 10, 10, 4, 17, 11, 7, 11, 11, 5,
        10, 4, 17, 17, 7, 11,
    ];

    cycles[opcode as usize]
}
