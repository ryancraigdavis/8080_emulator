// References - https://docs.rs/rodio/0.14.0/rodio/ 
// References - https://github.com/mohanson/space-invaders/
#[derive(Debug, Default)]
struct Invaderwavs {
    sink: rodio::Sink,
    wavs: [Vec<u8>; 10],
}

impl Invaderwavs {
    fn power_up(snd: impl AsRef<Path>) -> Self {
        let res = snd.as_ref().to_path_buf();
        let get = |x| {
            let mut res = res.clone();
            res.push(x);
            std::fs::read(res).unwrap()
        };
        let mut wavs = [
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        for i in 0..10 {
            wavs[i as usize] = get(format!("{}.wav", i));
        }
        let device = rodio::default_output_device().unwrap();
        let sink = rodio::Sink::new(&device);
        Self { sink, wavs }
    }

    fn play_sound(&self, i: usize) {
        let data = self.wavs[i].clone();
        let cursor = Cursor::new(data);
        self.sink.append(rodio::Decoder::new(cursor).unwrap());
    }
}
