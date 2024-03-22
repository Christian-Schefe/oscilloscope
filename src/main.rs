use std::{fs::File, io::Read, time::Duration};

use soundmaker::prelude::{flute, piano, violin, Vibrato, DAW, FM, SineSynth};

mod app;
mod fps;
mod line;
mod wave;

fn main() {
    let mut file = File::open("./assets/castle.mid").unwrap();

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let sample_rate = 48000.0;

    let mut daw = DAW::new();

    let flute = flute();
    let violin = violin();
    let piano = piano();

    let envelope = (0.03, 1.0, 0.8, 0.3);
    let vibrato = Vibrato::new(0.004, envelope, 5.0, envelope);
    let flute = FM::new(vibrato, envelope, (3.0, 1.0));

    // daw.add_channel("Sine".to_string(), SineSynth, vec![], 1.0, 0.0);
    // daw.add_channel("Sine".to_string(), SineSynth, vec![], 1.0, 0.0);
    // daw.add_channel("Sine".to_string(), SineSynth, vec![], 1.0, 0.0);
    // daw.add_channel("Sine".to_string(), SineSynth, vec![], 1.0, 0.0);
    // daw.add_channel("Sine".to_string(), SineSynth, vec![], 1.0, 0.0);

    daw.add_instrument("Piano RH".to_string(), &piano, 2.0, 0.0);
    daw.add_instrument("Piano LH".to_string(), &piano, 2.5, 0.0);

    daw.add_instrument("Violin".to_string(), &flute, 1.0, 0.0);
    daw.add_instrument("Flute".to_string(), &flute, 1.0, 0.0);
    daw.add_instrument("Violoncello".to_string(), &violin, 2.5, 0.0);

    daw.set_midi_bytes(&buffer);

    // daw.duration = Duration::from_secs(20);
    daw.master.volume = 1.0;

    app::run(daw, sample_rate);
}
