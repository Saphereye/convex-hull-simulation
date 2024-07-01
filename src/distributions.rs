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
    CircleArea,
    CirclePerimeter,
    SquareArea,
}

/// A resource that stores the current distribution type
#[derive(Resource)]
pub struct Distribution(pub DistributionType);

/// Stores the value for [Golden Angle](https://en.wikipedia.org/wiki/Golden_angle).
/// Used by [fibonacci_circle] function to generate points in a fibonacci spiral.
const GOLDEN_ANGLE: f32 = 2.3998277;

/// Generate a point in the fibonacci spiral at the given index
///
/// The functions generates points in this fashion. This method produces an 'even' distribution of points.
/// Atleast to a certain extent.
/// ![Example Image](https://i.stack.imgur.com/zcCaT.png)
///
/// Further reading
/// - [Why this produces evenly distributed points](https://math.stackexchange.com/questions/1934101/why-does-a-golden-angle-based-spiral-produce-evenly-distributed-points)
/// - [Extended to 3D](https://math.stackexchange.com/questions/3291489/can-the-fibonacci-lattice-be-extended-to-dimensions-higher-than-3)
/// - [Going a step beyond in 3D](https://extremelearning.com.au/how-to-evenly-distribute-points-on-a-sphere-more-effectively-than-the-canonical-fibonacci-lattice/)
pub fn fibonacci_circle(index: usize) -> (f32, f32) {
    let index: f32 = (index as f32) - (index as f32) / 2.0;

    let angle = 2.0 * std::f32::consts::PI * index * (1.0 / GOLDEN_ANGLE);
    let radius = 100.0 * (index - 0.5).sqrt();

    let x = (angle.cos() * radius).round();
    let y = (angle.sin() * radius).round();

    (x, y)
}

/// Generates a random point within a circle
/// 
/// Utilizes rejection sampling on the square area distribution to get
/// circular distribution.
pub fn circle_area(num_shapes: usize) -> (f32, f32) {
    let radius = 100.0 * (num_shapes as f32 - 0.5).sqrt();

    loop {
        let (x, y) = square_area(num_shapes);
        
        if (x*x + y*y)/radius <= radius {
            return (x, y);
        }
    }
}

/// Generates points on the perimeter of circle
///
/// The points are generated in this fashion
/// ![Example Image](https://mathworld.wolfram.com/images/eps-svg/CirclePointPicking_800.svg)
///
/// Further reading
/// - [Circle Point Picking](https://mathworld.wolfram.com/CirclePointPicking.html)
pub fn circle_perimeter(num_shapes: usize) -> (f32, f32) {
    let radius = 100.0 * (num_shapes as f32 - 0.5).sqrt();
    let angle: f32 = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let x = (angle.cos() * radius).round();
    let y = (angle.sin() * radius).round();

    (x, y)
}

/// Generates points inside a square
pub fn square_area(num_shapes: usize) -> (f32, f32) {
    let mut rng = thread_rng();
    let side_length = 2_f32 * (100_f32 * (num_shapes as f32 - 0.5).sqrt());

    let x: f32 = rng
        .gen_range(-(side_length/2.0)..(side_length/2.0))
        .round();
    let y: f32 = rng
        .gen_range(-(side_length/2.0)..(side_length/2.0))
        .round();

    (x, y)
}
