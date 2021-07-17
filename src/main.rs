pub mod disassembler;
pub mod opcodes;

fn main() {
    let code: [i32; 3] = [0x11, 0xf3, 0xd2];
    disassembler::disassembler_8080(&code);
    // match input {
    //     0x32 => println!("NOP"),
    //     _ => println!("Something else")
    // }
}
