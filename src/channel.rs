use std::time::Instant;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustfft::{num_complex::Complex32, num_traits::Zero, *};

use crate::{
    line::samples_to_path,
    wave::{PlaybackResource, WaveResource},
};

#[derive(Component)]
pub struct ChannelData {
    data: Vec<f64>,
    frame_indices: Vec<usize>,
    pub position: Rect,
    pub buffer_size: usize,
    pub target_fps: f64,
    prev: (usize, Vec<Complex32>),
    pub name: String,
}

impl ChannelData {
    pub fn new(
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
            prev: (2 * buffer_size, vec![Complex32::zero(); buffer_size]),
            name,
            target_fps,
        }
    }
    pub fn precompute_indices(&mut self, sample_rate: f64) {
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

            // let best_i = self.find_by_zero(index);
            let best_i = self.find_by_comp(800, index);
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
    fn find_by_comp(&mut self, samples_per_frame: usize, index: usize) -> usize {
        let data_diff = |i: usize, j: usize| -> f64 {
            (1..=self.buffer_size)
                .map(|x| (self.data[i - x] - self.data[j - x]).abs())
                .sum()
        };

        let zeros = (0..samples_per_frame).filter_map(|x| {
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
                let score = data_diff(self.prev.0, x);
                (score, x)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));

        if let Some((_, best_index)) = best {
            self.prev.0 = best_index;
        } else {
            self.prev.0 = index;
        }

        self.prev.0 + self.buffer_size / 2
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
    pub fn get_data(&self, frame: usize) -> &[f64] {
        let frame = frame.min(self.frame_indices.len() - 1);
        let i = self.frame_indices[frame];
        &self.data[i - self.buffer_size..i]
    }
}

pub fn update_channel(
    window: Query<&Window>,
    mut query: Query<(&mut ChannelData, &mut Path)>,
    playback: Res<PlaybackResource>,
) {
    let elapsed = playback.elapsed();
    let w = window.single();
    let width = w.width() as f32;
    let height = w.height() as f32;

    for (channel, mut path) in query.iter_mut() {
        let frame = (channel.target_fps * elapsed) as usize;
        let slice = channel.get_data(frame);

        let new_path = samples_to_path(slice, channel.position, width, height);
        *path = new_path;
    }
}

pub fn setup_channels(
    mut commands: Commands,
    wave: Res<WaveResource>,
    playback: Res<PlaybackResource>,
) {
    let channel_count = wave.channels.len();
    let y_spacing = 1.0 / (channel_count + 1) as f32;

    let mut channel_data: Vec<ChannelData> = wave
        .channels
        .iter()
        .enumerate()
        .map(|(i, channel)| {
            let min_y = (channel_count - i) as f32 * y_spacing;
            let max_y = (channel_count + 1 - i) as f32 * y_spacing;
            let rect = Rect::new(-0.5, min_y - 0.5, 0.5, max_y - 0.5);
            ChannelData::new(channel, wave.channel_names[i].clone(), rect, 4096, 60.0)
        })
        .collect();

    channel_data.push(ChannelData::new(
        &wave.master,
        "Master".to_string(),
        Rect::new(-0.5, -0.5, 0.5, y_spacing - 0.5),
        4096,
        60.0,
    ));

    channel_data
        .par_iter_mut()
        .for_each(|x| x.precompute_indices(playback.sample_rate));

    for (i, data) in channel_data.into_iter().enumerate() {
        let path = PathBuilder::new().build();
        let name = data.name.clone();

        commands.spawn((
            data,
            ShapeBundle {
                path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::hex("6cb8ff").unwrap(), 1.0),
            Fill::color(Color::NONE),
        ));

        let min_y = i as f32 * y_spacing;

        commands.spawn(TextBundle {
            text: Text::from_section(
                name,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(min_y * 100.0 + 1.3),
                left: Val::Px(10.0),
                ..Default::default()
            },

            ..default()
        });

        if i != 0 {
            let center_y = 100.0 * (i as f32) * y_spacing;
            commands.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Percent(center_y),
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::hex("444d56").unwrap()),
                ..default()
            });
        }
    }
}
