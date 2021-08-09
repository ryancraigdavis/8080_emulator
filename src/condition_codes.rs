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
        self.set_ac_flag(val);
    }

    pub fn set_inr_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
        self.set_ac_flag(val);
    }

    pub fn set_dcr_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
        self.set_ac_flag(val);
    }

    pub fn set_sub_flags(&mut self, val: u16) {
        self.set_zero_flag(val);
        self.set_sign_flag(val);
        self.set_carry_flag_sub(val);
        let lower = val as u8;
        self.set_parity_flag(lower);
        self.set_ac_flag(val);
    }

    pub fn set_zero_flag(&mut self, val: u16) {
        self.z = val & 0xff == 0;
    }

    pub fn set_sign_flag(&mut self, val: u16) {
        self.s = val & 0x80 == 0x80;
    }

    pub fn set_carry_flag_add(&mut self, val: u16) {
        self.cy = val > 0xff;
    }

    pub fn set_carry_flag_sub(&mut self, val: u16) {
        self.cy = val < 0x100;
    }

    pub fn set_parity_flag(&mut self, val: u8) {
        let one_count: u32 = val.count_ones();
        self.p = one_count & 1 == 0;
    }

    pub fn set_ac_flag(&mut self, val: u16) {
        self.ac = (val as u8) > 0xf;
    }
}
