//! Contains the different types of distributions that can be used to place the points
//! 
//! The distributions are:
//! - Fibonacci
//! - Random

use bevy::prelude::*;
use rand::{thread_rng, Rng};
    
/// The different types of distributions that can be used to place the points
#[derive(PartialEq, Clone, Copy)]
pub enum DistributionType {
    Fibonacci,
    Random,
    CirclePerimeter,
    Square,
}

/// A resource that stores the current distribution type
#[derive(Resource)]
pub struct Distribution(pub DistributionType);

/// Stores the value for [Golden Angle](https://en.wikipedia.org/wiki/Golden_angle).
/// Used by [fibonacci_circle] function to generate points in a fibonacci spiral.
const GOLDEN_ANGLE: f32 = 2.3998277;

/// Generate a point in the fibonacci spiral at the given index
pub fn fibonacci_circle(index: usize) -> (f32, f32) {
    let index: f32 = (index as f32) - (index as f32) / 2.0;

    let angle = 2.0 * std::f32::consts::PI * index * (1.0 / GOLDEN_ANGLE);
    let radius = 100.0 * (index - 0.5).sqrt();

    let x = (angle.cos() * radius).round();
    let y = (angle.sin() * radius).round();

    (x, y)
}

/// Generates a random point within a circle with a radius of `num_shapes * 25.0`
pub fn bounded_random(num_shapes: usize) -> (f32, f32) {
    let radius = num_shapes as f32 * 25.0 * rand::random::<f32>();
    let angle: f32 = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let x = (angle.cos() * radius).round();
    let y = (angle.sin() * radius).round();

    (x, y)
}

/// Generates points on the perimeter of circle
pub fn circle_points(num_shapes: usize) -> (f32, f32) {
    let radius = num_shapes as f32 * 5.0;
    let angle: f32 = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let x = (angle.cos() * radius).round();
    let y = (angle.sin() * radius).round();

    (x, y)
}

/// Generates points inside a square
pub fn bounded_random_square(num_shapes: usize) -> (f32, f32) {
    let mut rng = thread_rng();

    let x: f32 = rng.gen_range(-(num_shapes as f32 * 5.0)..(num_shapes as f32 * 5.0)).round();
    let y: f32 = rng.gen_range(-(num_shapes as f32 * 5.0)..(num_shapes as f32 * 5.0)).round();

    (x, y)
}
