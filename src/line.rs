use bevy::prelude::*;
use bevy_prototype_lyon::{entity::Path, path::PathBuilder};
use geo::{simplify::*, Coord, LineString};

pub fn samples_to_path(samples: &[f64], rect: Rect, width: f32, height: f32) -> Path {
    let sample_count = samples.len() as f32;
    let points: Vec<Vec2> = samples
        .iter()
        .enumerate()
        .map(|(i, &s)| Vec2::new(i as f32, (s as f32 * 0.5 + 0.5) * sample_count))
        .collect();

    let points = simplify_points(points, 0.5);

    // info!("simplified {} to {} points", samples.len(), points.len());
    let points = position_points(points, sample_count, rect, width, height);

    points_to_path(points)
}

fn position_points(
    points: Vec<Vec2>,
    sample_count: f32,
    rect: Rect,
    width: f32,
    height: f32,
) -> Vec<Vec2> {
    let x_factor = 1.0 / (sample_count - 1.0);
    let y_factor = 1.0 / sample_count;
    points
        .into_iter()
        .map(|p| {
            lerp_rect(Vec2::new(p.x * x_factor, p.y * y_factor), rect) * Vec2::new(width, height)
        })
        .collect()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_rect(t: Vec2, rect: Rect) -> Vec2 {
    Vec2::new(
        lerp(rect.min.x, rect.max.x, t.x),
        lerp(rect.min.y, rect.max.y, t.y),
    )
}

fn simplify_points(points: Vec<Vec2>, tolerance: f32) -> Vec<Vec2> {
    let points = points
        .into_iter()
        .map(|p| Coord { x: p.x, y: p.y })
        .collect();

    let line = LineString::new(points);
    let line = line.simplify(&tolerance);

    let points = line.points().map(|p| Vec2::new(p.0.x, p.0.y));
    points.collect()
}

fn points_to_path(points: Vec<Vec2>) -> Path {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(points[0]);
    for point in points.into_iter().skip(1) {
        path_builder.line_to(point);
    }
    path_builder.build()
}
