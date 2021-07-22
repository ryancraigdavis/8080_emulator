// References - http://www.emulator101.com/emulator-shell.html
#[derive(Debug, Default)]
pub struct ConditionCodes {
    pub z: bool,
    pub s: bool,
    pub p: bool,
    pub cy: bool,
    pub ac: bool,
    pub pad: u8,
}
