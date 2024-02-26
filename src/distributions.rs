use bevy::prelude::*;

#[derive(PartialEq)]
pub enum DistributionType {
    Fibonacci,
    Random,
}

#[derive(Resource)]
pub struct Distribution(pub DistributionType);

const GOLDEN_ANGLE: f32 = 2.3998277;

pub fn fibonacci_circle(index: usize) -> (f32, f32) {
    let index: f32 = (index as f32) - (index as f32) / 2.0;

    let angle = 2.0 * std::f32::consts::PI * index * (1.0 / GOLDEN_ANGLE);
    let radius = 100.0 * (index - 0.5).sqrt();

    let x = angle.cos() * radius;
    let y = angle.sin() * radius;

    (x, y)
}

pub fn bounded_random(num_shapes: usize) -> (f32, f32) {
    let radius = num_shapes as f32 * 25.0 * rand::random::<f32>();
    let angle: f32 = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let x = angle.cos() * radius;
    let y = angle.sin() * radius;

    (x, y)
}
