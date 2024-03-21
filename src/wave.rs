use std::time::Instant;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustfft::{num_complex::Complex32, num_traits::Zero, *};
use soundmaker::daw::RenderedAudio;

pub struct WavePlugin(pub f64);

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlaybackResource::new(self.0))
            .add_systems(Startup, (setup_channels, start_playback).chain())
            .add_systems(Update, update_channel);
    }
}

#[derive(Resource)]
pub struct WaveResource {
    master: Vec<(f64, f64)>,
    channels: Vec<Vec<(f64, f64)>>,
}

impl WaveResource {
    pub fn new(master: Vec<(f64, f64)>, channels: Vec<Vec<(f64, f64)>>) -> Self {
        Self { master, channels }
    }
}

impl From<RenderedAudio> for WaveResource {
    fn from(render: RenderedAudio) -> Self {
        Self::new(render.master, render.channels)
    }
}

fn setup_channels(
    mut commands: Commands,
    wave: Res<WaveResource>,
    playback: Res<PlaybackResource>,
) {
    let channel_count = wave.channels.len();
    let mut channel_data: Vec<ChannelData> = wave
        .channels
        .iter()
        .enumerate()
        .map(|(i, channel)| {
            let min_y = i as f32 / channel_count as f32;
            let max_y = (i + 1) as f32 / channel_count as f32;
            let rect = Rect::new(0.0, min_y, 1.0, max_y);
            ChannelData::new(channel, "Name".to_string(), rect, 4096, 60.0)
        })
        .collect();

    channel_data
        .par_iter_mut()
        .for_each(|x| x.precompute_indices(playback.sample_rate));

    for data in channel_data {
        let path = PathBuilder::new().build();

        commands.spawn((
            data,
            ShapeBundle {
                path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0., 75., 0.),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::BLACK, 1.0),
            Fill::color(Color::NONE),
        ));
    }
}

#[derive(Component)]
pub struct ChannelData {
    data: Vec<f64>,
    frame_indices: Vec<usize>,
    position: Rect,
    buffer_size: usize,
    target_fps: f64,
    prev: (usize, Vec<Complex32>),
    name: String,
}

impl ChannelData {
    fn new(
        data: &[(f64, f64)],
        name: String,
        position: Rect,
        buffer_size: usize,
        target_fps: f64,
    ) -> Self {
        Self {
            data: vec![0.0; buffer_size * 2]
                .into_iter()
                .chain(data.into_iter().map(|x| (x.0 + x.1) / 2.0))
                .chain(vec![0.0; buffer_size * 2])
                .collect(),
            frame_indices: Vec::new(),
            position,
            buffer_size,
            prev: (0, vec![Complex32::zero(); buffer_size]),
            name,
            target_fps,
        }
    }
    fn precompute_indices(&mut self, sample_rate: f64) {
        let mut indices = Vec::new();
        let secs_per_frame = 1.0 / self.target_fps;
        let mut i = 0;
        println!("Precomputing {}...", self.name);
        let start_time = Instant::now();
        loop {
            let passed_time = i as f64 * secs_per_frame;
            let index = 2 * self.buffer_size + (sample_rate * passed_time) as usize;
            if index > self.data.len() {
                indices.push(self.data.len()); // Last Frame is just zeros
                break;
            }

            let best_i = self.find_by_zero(index);
            let clamped_i = best_i.clamp(self.buffer_size * 2, self.data.len());
            indices.push(clamped_i);
            i += 1;
        }
        println!(
            "Finished precomputing {} in {:.2}s",
            self.name,
            start_time.elapsed().as_secs_f32()
        );
        self.frame_indices = indices;
    }
    fn find_by_zero(&mut self, index: usize) -> usize {
        let zeros = (0..800).filter_map(|x| {
            let i = index - x;
            let val = self.data[i];
            if val >= 0.0 && self.data[i - 1] < 0.0 {
                Some(i)
            } else {
                None
            }
        });

        let best = zeros
            .map(|x| {
                let spectrum = Self::perform_fft(&self.data[x - self.buffer_size..x]);
                let score = Self::cross_correlation(&self.prev.1, &spectrum);
                (score, x, spectrum)
            })
            .max_by(|a, b| a.0.total_cmp(&b.0));

        if let Some((_, best_index, best_spectrum)) = best {
            self.prev = (best_index, best_spectrum);
        } else {
            self.prev = (
                index,
                Self::perform_fft(&self.data[index - self.buffer_size..index]),
            );
        }

        self.prev.0 + self.buffer_size / 2
    }
    fn cross_correlation(prev_spectrum: &[Complex32], spectrum: &[Complex32]) -> f32 {
        let mut cross_correlation = 0.0;
        let window_size = prev_spectrum.len().min(spectrum.len());
        for i in 0..window_size {
            cross_correlation +=
                prev_spectrum[i].re * spectrum[i].re + prev_spectrum[i].im * spectrum[i].im;
        }
        cross_correlation /= window_size as f32;
        cross_correlation
    }
    fn perform_fft(samples: &[f64]) -> Vec<Complex32> {
        let length = samples.len();

        let mut spectrum: Vec<Complex32> = samples
            .into_iter()
            .map(|x| Complex32 {
                re: *x as f32,
                im: 0.0,
            })
            .collect();

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(length);
        fft.process(&mut spectrum);
        spectrum
    }
    fn get_data(&self, frame: usize) -> &[f64] {
        let i = self.frame_indices[frame];
        &self.data[i - self.buffer_size..i]
    }
}

#[derive(Resource)]
pub struct PlaybackResource {
    sample_rate: f64,
    start_instant: Instant,
}

impl PlaybackResource {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            start_instant: Instant::now(),
        }
    }
    pub fn start(&mut self) {
        self.start_instant = Instant::now();
    }
    pub fn elapsed(&self) -> f64 {
        self.start_instant.elapsed().as_secs_f64()
    }
}

pub fn update_channel(
    window: Query<&Window>,
    mut query: Query<(&mut ChannelData, &mut Path)>,
    playback: Res<PlaybackResource>,
) {
    let elapsed = playback.elapsed();
    let w = window.single();
    let width = w.width() as f64;
    let height = w.height() as f64;

    for (mut channel, mut path) in query.iter_mut() {
        let frame = (channel.target_fps * elapsed) as usize;
        let slice = channel.get_data(frame);
        let sample_count = slice.len();
        let points = slice
            .iter()
            .enumerate()
            .map(|(i, y)| position_point01(sample_count, *y, i))
            .collect();

        let new_path = build_path(points, channel.position, width, height);
        *path = new_path;
    }
}

fn position_point01(sample_count: usize, sample: f64, index: usize) -> Vec2 {
    let x = index as f64 / (sample_count - 1) as f64;
    let y = sample * 0.5 + 0.5;
    Vec2::new(x as f32, y as f32)
}

fn build_path(points: Vec<Vec2>, rect: Rect, width: f64, height: f64) -> Path {
    let mut new_points = points.into_iter().map(|point| {
        let x = (rect.min.x + rect.width() * point.x) * width as f32;
        let y = (rect.min.y + rect.height() * point.y) * height as f32;
        Vec2::new(x, y)
    });

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(new_points.next().unwrap());
    for point in new_points {
        path_builder.line_to(point);
    }
    let path = path_builder.build();
    path
}

fn start_playback(mut playback: ResMut<PlaybackResource>) {
    playback.start();
}
