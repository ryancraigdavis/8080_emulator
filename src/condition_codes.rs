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

impl ConditionCodes {
    pub fn set_zero_flag(&mut self, val: u16) {
        if val & 0xff == 0 {
            self.z = true;
        } else {
            self.z = false;
        }
    }

    pub fn set_sign_flag(&mut self, val: u16) {
        if val & 0x80 == 0x80 {
            self.s = true;
        } else {
            self.s = false;
        }
    }

    pub fn set_carry_flag(&mut self, val: u16) {
        if val > 0xff {
            self.cy = true;
        } else {
            self.cy = false;
        }
    }

    pub fn set_parity_flag(&mut self, val: u8) {
        let lower = val as u8;
        let one_count: u32 = lower.count_ones();
        if one_count & 1 == 0 {
            self.p = true;
        } else {
            self.p = false;
        }
    }
}
