use std::path::PathBuf;

use bevy::{prelude::*, window::PresentMode};
use bevy_prototype_lyon::prelude::*;
use soundmaker::daw::{render_daw, RenderedAudio, DAW};

use serde::{Deserialize, Serialize};

use crate::{
    fps::FpsDiagnosticsPlugin,
    wave::{WavePlugin, WaveResource},
};

pub fn run(mut daw: DAW, sample_rate: f64) {
    let render = get_render(&mut daw, sample_rate, PathBuf::from("./output/render.bin"));

    App::new()
        .insert_resource(ClearColor(Color::hex("24292e").unwrap()))
        .insert_resource(WaveResource::from((render, daw)))
        .add_plugins(WavePlugin(sample_rate))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Oscilloscope".to_string(),
                present_mode: PresentMode::Fifo,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .add_plugins(FpsDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Serialize, Deserialize)]
pub struct DataStorage {
    pub master: Vec<(f64, f64)>,
    pub channels: Vec<Vec<(f64, f64)>>,
}

fn get_render(daw: &mut DAW, sample_rate: f64, file_path: PathBuf) -> RenderedAudio {
    if let Ok(bytes) = std::fs::read(file_path.clone()) {
        let decoded: DataStorage = bincode::deserialize(&bytes).unwrap();
        info!("Loaded render from file: {:?}", file_path);

        RenderedAudio {
            master: decoded.master,
            channels: decoded.channels,
        }
    } else {
        let data = render_daw(daw, sample_rate);
        let storage = DataStorage {
            master: data.master.clone(),
            channels: data.channels.clone(),
        };
        let encoded: Vec<u8> = bincode::serialize(&storage).unwrap();
        std::fs::write(file_path.clone(), encoded).unwrap();
        info!("Saved render to file: {:?}", file_path);
        data
    }
}
