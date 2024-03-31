use soundmaker::prelude::*;

mod app;
mod fps;
mod line;
mod wave;
mod channel;

fn main() {
    let sample_rate = find_sample_rate();

    // let mut daw = chill_beats();
    let mut daw = castle();
    daw.master.volume = 1.0;

    app::run(daw, sample_rate);
}

fn castle() -> DAW {
    let bytes = std::fs::read("./assets/castle.mid").unwrap();

    let mut daw = DAW::new();

    let violin = violin();
    let piano = piano();

    daw.add_instrument("Piano RH".to_string(), &piano, 1.0, 0.0);
    daw.add_instrument("Piano LH".to_string(), &piano, 1.8, 0.0);

    daw.add_instrument("Violin".to_string(), &violin, 1.0, 0.0);
    daw.add_instrument("Flute".to_string(), &violin, 1.0, 0.0);
    daw.add_instrument("Violoncello".to_string(), &violin, 1.0, 0.0);

    daw.set_midi_bytes(&bytes);
    daw
}

fn chill_beats() -> DAW {
    let midi = std::fs::read("./assets/Chill Beats.mid").unwrap();
    let mut daw = DAW::new();

    let violin = violin();
    let flute = flute();

    let percussion = percussion(vec![
        Percussion::BassDrum(36, 0.4),
        Percussion::SnareDrum(38, 0.7),
        Percussion::HiHat(44, 1.0),
        Percussion::Shaker(70, 1.0),
    ]);

    daw.add_instrument("Flute".to_string(), &flute, 1.0, 0.0);
    daw.add_instrument("Percussion 1".to_string(), percussion.as_ref(), 1.0, 0.0);
    daw.add_instrument("Percussion 2".to_string(), percussion.as_ref(), 1.0, 0.0);
    daw.add_instrument("Viola".to_string(), &violin, 1.0, 0.0);
    daw.add_instrument("Cello".to_string(), &violin, 1.0, 0.0);

    daw.set_midi_bytes(&midi);
    daw
}
