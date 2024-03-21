use std::{fs::File, io::Read, time::Duration};

use soundmaker::prelude::{piano, violin, DAW};

mod app;
mod fps;
mod line;
mod wave;

fn main() {
    let mut file = File::open("./assets/Dream Of The Ocean.mid").unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let sample_rate = 48000.0;

    let mut daw = DAW::new();

    let violin = violin();
    daw.add_instrument("Violin".to_string(), &violin, 2.5, 0.0);
    daw.add_instrument("Violoncello".to_string(), &violin, 2.5, 0.0);

    let piano = piano();
    daw.add_instrument("Piano LH".to_string(), &piano, 2.0, 0.0);
    daw.add_instrument("Piano RH".to_string(), &piano, 2.5, 0.0);

    daw.set_midi_bytes(&buffer);

    daw.duration = Duration::from_secs(10);
    daw.master.volume = 0.0;

    app::run(daw, sample_rate);
}
