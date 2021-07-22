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
}
