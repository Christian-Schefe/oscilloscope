use bevy::{prelude::*, window::PresentMode};
use bevy_prototype_lyon::prelude::*;
use soundmaker::daw::{render_daw, DAW};

use crate::{
    fps::FpsDiagnosticsPlugin,
    wave::{WavePlugin, WaveResource},
};

pub fn run(mut daw: DAW, sample_rate: f64) {
    let render = render_daw(&mut daw, sample_rate);
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
