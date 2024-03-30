use std::{
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use bevy::{input::keyboard::KeyboardInput, prelude::*, window::close_on_esc};
use soundmaker::prelude::*;

use crate::channel::*;
use std::thread;

pub struct WavePlugin(pub f64);

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlaybackResource::new(self.0))
            .add_systems(Startup, (setup_channels, start_playback).chain())
            .add_systems(Update, close_on_esc)
            .add_systems(Update, update_channel)
            .add_systems(Update, handle_pause_playback);
    }
}

#[derive(Resource)]
pub struct WaveResource {
    pub master: Vec<(f64, f64)>,
    pub channels: Vec<Vec<(f64, f64)>>,
    pub channel_names: Vec<String>,
}

impl WaveResource {
    pub fn new(
        master: Vec<(f64, f64)>,
        channels: Vec<Vec<(f64, f64)>>,
        channel_names: Vec<String>,
    ) -> Self {
        Self {
            master,
            channels,
            channel_names,
        }
    }
}

impl From<(RenderedAudio, DAW)> for WaveResource {
    fn from(data: (RenderedAudio, DAW)) -> Self {
        let channel_names = (0..data.1.channel_count)
            .map(|i| data.1[i].name.clone())
            .collect();
        Self::new(data.0.master, data.0.channels, channel_names)
    }
}

#[derive(Resource)]
pub struct PlaybackResource {
    pub sample_rate: f64,
    start_instant: Option<Instant>,
    controller: Option<(Shared<f32>, Shared<f64>, Shared<f64>)>,
    paused_time: Option<f64>,
}

impl PlaybackResource {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            start_instant: None,
            controller: None,
            paused_time: None,
        }
    }
    pub fn elapsed(&self) -> f64 {
        if let Some(paused_time) = self.paused_time {
            paused_time
        } else if let Some(start_time) = self.start_instant {
            start_time.elapsed().as_secs_f64()
        } else {
            0.0
        }
    }
    pub fn pause(&mut self) {
        if let Some(controls) = &self.controller {
            controls.0.set(-1.0);
            self.paused_time = Some(self.elapsed());
        }
    }
    pub fn unpause(&mut self) {
        if let Some(controls) = &self.controller {
            controls.0.set(1.0);
            self.start_instant =
                Some(Instant::now() - Duration::from_secs_f64(self.paused_time.unwrap()));
            self.paused_time = None;
        }
    }
    pub fn toggle_pause(&mut self) {
        if self.paused_time.is_some() {
            self.unpause();
        } else {
            self.pause();
        }
    }
    pub fn set_time(&mut self, time: f64) {
        if let Some(controls) = &self.controller {
            controls.1.set(time);
            self.start_instant = Some(Instant::now() - Duration::from_secs_f64(time));
            if self.paused_time.is_some() {
                self.paused_time = Some(time);
            }
        }
    }
    pub fn mul_volume(&self, factor: f64) {
        if let Some(controls) = &self.controller {
            let new_volume = controls.2.value() * factor;
            controls.2.set(new_volume.max(0.0))
        }
    }
}

fn start_playback(mut playback: ResMut<PlaybackResource>, data: Res<WaveResource>) {
    let data = data.master.clone();
    let sample_rate = playback.sample_rate;

    let (tx, rx) = channel();

    thread::spawn(move || {
        play_and_save(data, sample_rate, "./output/output.wav".into(), tx).unwrap();
    });

    let (start_instant, controls) = rx.recv().unwrap();

    playback.start_instant = Some(start_instant);
    playback.controller = Some(controls);
}

fn handle_pause_playback(
    mut playback: ResMut<PlaybackResource>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
) {
    for event in keyboard_input_events.read() {
        if event.state.is_pressed() {
            if event.key_code == KeyCode::Space {
                playback.toggle_pause();
            }
            if event.key_code == KeyCode::KeyR {
                playback.set_time(0.0)
            }
            if event.key_code == KeyCode::ArrowUp {
                playback.mul_volume(1.5);
                info!(
                    "Volume: {}",
                    playback.controller.as_ref().unwrap().2.value()
                );
            }
            if event.key_code == KeyCode::ArrowDown {
                playback.mul_volume(1.0 / 1.5);
                info!(
                    "Volume: {}",
                    playback.controller.as_ref().unwrap().2.value()
                );
            }
        }
    }
}
