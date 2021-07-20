pub mod disassembler;
use std::fs;
use std::path::Path;

fn main() {
    let code: [i32; 3] = [0x11, 0xf3, 0xd2];

    let path = Path::new("space_invaders_rom/invaders.h");
    let contents = fs::read_to_string(path);

    println!("{}", contents[0]);

    let mut opbytes: i32 = disassembler::disassembler_8080(&code, 0);
}
