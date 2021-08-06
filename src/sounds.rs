// References - https://docs.rs/rodio/0.14.0/rodio/
// References - https://github.com/mohanson/space-invaders/
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
}
