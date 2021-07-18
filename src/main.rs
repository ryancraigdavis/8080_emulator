pub mod disassembler;

fn main() {
    let code: [i32; 3] = [0x11, 0xf3, 0xd2];
    let mut opbytes: i32 = disassembler::disassembler_8080(&code, 0);
}
