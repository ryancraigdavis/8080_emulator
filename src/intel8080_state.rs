use crate::condition_codes::ConditionCodes;

// Created using http://www.emulator101.com/emulator-shell.html as a resource
#[derive(Debug, Default)]
pub struct StateIntel8080 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub condition: ConditionCodes,
    pub memory: Vec<u8>,
}

impl StateIntel8080 {
    pub fn init_mem(&mut self, buf: &mut Vec<u8>) {
        // intel 8080 has a maximum memory of 64KB
        self.memory = vec![0; 0xffff];
        let mut i = 0;
        while i < buf.len() {
            self.memory[i] = buf[i];
            i += 1;
        }
        // self.memory = buf.clone();
        // self.memory = vec![0; 0x4000];
    }
}
