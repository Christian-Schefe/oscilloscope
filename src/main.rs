use soundmaker::prelude::{flute, piano, violin, DAW};

mod app;
mod fps;
mod line;
mod wave;

fn main() {
    // let mut file = File::open("./assets/Dream Of The Ocean.mid").unwrap();
    let bytes = std::fs::read("./assets/castle.mid").unwrap();

    let sample_rate = 48000.0;

    let mut daw = DAW::new();

    let flute = flute();
    let violin = violin();
    let piano = piano();

    daw.add_instrument("Piano RH".to_string(), &piano, 2.5, 0.0);
    daw.add_instrument("Piano LH".to_string(), &piano, 4.5, 0.0);

    daw.add_instrument("Violin".to_string(), &violin, 2.5, 0.0);
    daw.add_instrument("Flute".to_string(), &violin, 2.5, 0.0);
    daw.add_instrument("Violoncello".to_string(), &violin, 2.5, 0.0);

    daw.set_midi_bytes(&bytes);

    // daw.duration = Duration::from_secs(20);
    daw.master.volume = 1.0;

    app::run(daw, sample_rate);
}
