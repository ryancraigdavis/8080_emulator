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
    pub fn set_add_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        self.set_carry_flag_add(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
    }

    pub fn set_inr_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
    }

    pub fn set_dcr_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
    }

    pub fn set_sub_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        self.set_carry_flag_sub(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
    }

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

    pub fn set_carry_flag_add(&mut self, val: u16) {
        if val > 0xff {
            self.cy = true;
        } else {
            self.cy = false;
        }
    }

    pub fn set_carry_flag_sub(&mut self, val: u16) {
        if val < 0x100 {
            self.cy = true;
        } else {
            self.cy = false;
        }
    }

    pub fn set_parity_flag(&mut self, val: u8) {
        let one_count: u32 = val.count_ones();
        if one_count & 1 == 0 {
            self.p = true;
        } else {
            self.p = false;
        }
    }
}
