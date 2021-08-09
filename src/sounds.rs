// References - https://docs.rs/rodio/0.14.0/rodio/
// References - https://github.com/mohanson/space-invaders/
// References - https://github.com/mohanson/i8080/blob/master/src/bit.rs
use rodio::source::Source;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Default)]
pub struct Invaderwavs {
    pub sounds: Vec<String>,
}

impl Invaderwavs {
    pub fn load_sounds(&mut self) {
        self.sounds.push(String::from("sounds/res_snd_0.wav"));
        self.sounds.push(String::from("sounds/res_snd_1.wav"));
        self.sounds.push(String::from("sounds/res_snd_2.wav"));
        self.sounds.push(String::from("sounds/res_snd_3.wav"));
        self.sounds.push(String::from("sounds/res_snd_4.wav"));
        self.sounds.push(String::from("sounds/res_snd_5.wav"));
        self.sounds.push(String::from("sounds/res_snd_6.wav"));
        self.sounds.push(String::from("sounds/res_snd_7.wav"));
        self.sounds.push(String::from("sounds/res_snd_8.wav"));
        self.sounds.push(String::from("sounds/res_snd_9.wav"));
    }
    pub fn play_sound(&self, i: usize) {
        let stream_handle = rodio::default_output_device().unwrap();
        let data = self.sounds[i].clone();
        let file = File::open(data).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        rodio::play_raw(&stream_handle, source.convert_samples());
    }
    pub fn get_sound_bit(&self, n: u8, b: usize) -> bool {
        (n & (1 << b)) != 0
    }
    pub fn queued_event(&self, reg_a: u8, event_type: u8, output_state: u8) -> bool {
        let mut success_bool: bool = false;
        if event_type == 1 && reg_a != output_state {
            if self.get_sound_bit(reg_a, 0) && !self.get_sound_bit(output_state, 0) {
                self.play_sound(0)
            }
            if self.get_sound_bit(reg_a, 1) && !self.get_sound_bit(output_state, 1) {
                self.play_sound(1)
            }
            if self.get_sound_bit(reg_a, 2) && !self.get_sound_bit(output_state, 2) {
                self.play_sound(2)
            }
            if self.get_sound_bit(reg_a, 3) && !self.get_sound_bit(output_state, 3) {
                self.play_sound(3)
            }
            success_bool = true;
        }
        if event_type == 2 && reg_a != output_state {
            if self.get_sound_bit(reg_a, 0) && !self.get_sound_bit(output_state, 0) {
                self.play_sound(4)
            }
            if self.get_sound_bit(reg_a, 1) && !self.get_sound_bit(output_state, 1) {
                self.play_sound(5)
            }
            if self.get_sound_bit(reg_a, 2) && !self.get_sound_bit(output_state, 2) {
                self.play_sound(6)
            }
            if self.get_sound_bit(reg_a, 3) && !self.get_sound_bit(output_state, 3) {
                self.play_sound(7)
            }
            if self.get_sound_bit(reg_a, 4) && !self.get_sound_bit(output_state, 4) {
                self.play_sound(8)
            }
            success_bool = true;
        }
        success_bool
    }
}
