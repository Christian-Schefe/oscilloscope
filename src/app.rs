use bevy::{prelude::*, window::PresentMode};
use bevy_prototype_lyon::prelude::*;
use soundmaker::daw::RenderedAudio;

use crate::{
    fps::FpsDiagnosticsPlugin,
    wave::{WavePlugin, WaveResource},
};

pub fn run(render: RenderedAudio, sample_rate: f64) {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(WaveResource::from(render))
        .add_plugins(WavePlugin(sample_rate))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Oscilloscope".to_string(),
                present_mode: PresentMode::Mailbox,
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
