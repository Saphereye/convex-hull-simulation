use std::cmp::Ordering;
use std::hash::Hash;

use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use std::collections::HashSet;

/// (is_drawing, number of iterations reached, initial state of the drawing)
#[derive(Resource)]
pub struct DrawingInProgress(pub bool, pub usize, pub usize);

#[derive(Component)]
pub struct Gizmo;

#[derive(Component)]
pub struct ConvexHull;

#[derive(PartialEq, Clone, Copy)]
pub enum AlgorithmType {
    JarvisMarch,
    KirkPatrickSeidel,
}

#[derive(Resource)]
pub struct Algorithm(pub AlgorithmType);

#[derive(Debug, PartialOrd, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

impl Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x) < f32::EPSILON && (self.y - other.y) < f32::EPSILON
    }
}

impl Eq for Point {}

macro_rules! draw_lines {
    ($commands:expr, $meshes:expr, $materials:expr, $line:expr, Gizmo) => {
        $commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(
                    $meshes.add(
                        Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, $line),
                    ),
                ),
                material: $materials.add(Color::rgb(0.44, 0.44, 0.44)),
                ..default()
            },
            Gizmo,
        ));
    };
    ($commands:expr, $meshes:expr, $materials:expr, $line:expr, ConvexHull) => {
        $commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(
                    $meshes.add(
                        Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, $line),
                    ),
                ),
                material: $materials.add(Color::rgb(1.0, 1.0, 1.0)),
                ..default()
            },
            ConvexHull,
        ));
    };
}

/// # Implementation of the [Jarvis March](https://en.wikipedia.org/wiki/Gift_wrapping_algorithm) algorithm
/// This algorithm is used to calculate the convex hull of given set of points.
/// It has a `O(nh)` time complexity, where `n` is the number of points and `h` is the number of points on the convex hull.
/// <p><a href="https://commons.wikimedia.org/wiki/File:Animation_depicting_the_gift_wrapping_algorithm.gif#/media/File:Animation_depicting_the_gift_wrapping_algorithm.gif"><img src="https://upload.wikimedia.org/wikipedia/commons/9/9c/Animation_depicting_the_gift_wrapping_algorithm.gif" alt="Animation depicting the gift wrapping algorithm.gif" height="401" width="401"></a><br></p>
///
/// ## Pseudocode
/// ```pseudocode
/// algorithm jarvis(S) is
///     # S is the set of points
///     # P will be the set of points which form the convex hull. Final set size is i.
///     pointOnHull = leftmost point in S // which is guaranteed to be part of the CH(S)
///     i := 0
///     repeat
///         P[i] := pointOnHull
///         endpoint := S[0]      // initial endpoint for a candidate edge on the hull
///         for j from 0 to |S| do
///             # endpoint == pointOnHull is a rare case and can happen only when j == 1 and a better endpoint has not yet been set for the loop
///             if (endpoint == pointOnHull) or (S[j] is on left of line from P[i] to endpoint) then
///                 endpoint := S[j]   // found greater left turn, update endpoint
///         i := i + 1
///         pointOnHull = endpoint
///     until endpoint = P[0]      // wrapped around to first hull point
/// ```
pub fn jarvis_march(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    points: Vec<Point>,
    current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    let mut current = current;
    let previous_point = [points[current].x, points[current].y, 0.0];
    let mut next = (current + 1) % points.len();
    for (i, point) in points.iter().enumerate() {
        draw_lines!(
            commands,
            meshes,
            materials,
            vec![
                [previous_point[0], previous_point[1], 0.0],
                [point.x, point.y, 0.0],
            ],
            Gizmo
        );

        if i != current && i != next {
            let cross = (point.x - points[current].x)
                * (points[next].y - points[current].y)
                - (point.y - points[current].y)
                    * (points[next].x - points[current].x);
            if cross < 0.0 {
                next = i;
            }
        }
    }

    draw_lines!(
        commands,
        meshes,
        materials,
        vec![
            [previous_point[0], previous_point[1], 0.0],
            [points[next].x, points[next].y, 0.0],
        ],
        ConvexHull
    );

    current = next;
    drawing_in_progress.1 = current;
    if current == drawing_in_progress.2 {
        drawing_in_progress.0 = false;
        drawing_in_progress.1 = 0;
    }
}

fn connect(lower: Point, upper: Point, points: Vec<Point>) -> Vec<Point> {
    if lower == upper {
        return vec![lower];
    }
    let max_left = quickselect(
        points.clone(),
        (points.len() / 2).saturating_sub(1),
        0,
        points.len() - 1,
    );
    let min_right = quickselect(points.clone(), points.len() / 2, 0, points.len() - 1);
    let (left, right) = bridge(
        points.clone().into_iter().collect(),
        (max_left.x + min_right.x) / 2.0,
    );
    let points_left: Vec<Point> = vec![left]
        .iter()
        .cloned()
        .chain(
            points
                .iter()
                .filter(|&point| point.x < left.x)
                .cloned(),
        )
        .collect();
    let points_right: Vec<Point> = vec![right]
        .iter()
        .cloned()
        .chain(
            points
                .iter()
                .filter(|&point| point.x > right.x)
                .cloned(),
        )
        .collect();
    let mut result = connect(lower, left, points_left);
    result.extend(connect(right, upper, points_right));
    result
}

fn quickselect<T: PartialOrd + Copy>(mut points: Vec<T>, index: usize, lo: usize, hi: usize) -> T {
    if lo == hi {
        return points[lo];
    }
    let pivot = lo + (hi - lo) / 2;
    points.swap(pivot, lo);
    let mut cur = lo;
    for run in lo + 1..=hi {
        if points[run] < points[lo] {
            cur += 1;
            points.swap(cur, run);
        }
    }
    points.swap(cur, lo);
    if index < cur {
        quickselect(points, index, lo, cur - 1)
    } else if index > cur {
        quickselect(points, index, cur + 1, hi)
    } else {
        points[cur]
    }
}

fn bridge(points: HashSet<Point>, vertical_line: f32) -> (Point, Point) {
    let mut candidates = HashSet::new();
    if points.len() == 2 {
        let points_vec: Vec<_> = points.into_iter().collect();
        return (points_vec[0], points_vec[1]);
    }
    let mut pairs = Vec::new();
    let mut modify_s = points.clone();

    while modify_s.len() >= 2 {
        let p1 = modify_s.iter().next().cloned().unwrap();
        modify_s.remove(&p1);
        let p2 = modify_s.iter().next().cloned().unwrap();
        modify_s.remove(&p2);
        pairs.push((p1, p2));
    }

    if modify_s.len() == 1 {
        candidates.insert(modify_s.iter().next().cloned().unwrap());
    }

    let mut slopes = Vec::new();
    let mut new_pairs = Vec::new();
    for (pi, pj) in pairs.iter() {
        if pi.x == pj.x {
            candidates.insert(if pi.y > pj.y { *pi } else { *pj });
        } else {
            slopes.push((pi.y - pj.y) / (pi.x - pj.x));
            new_pairs.push((*pi, *pj)); // Keep the pair if x values are not equal
        }
    }
    pairs = new_pairs; // Update pairs vector with the filtered pairs

    slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let median_index = (slopes.len() / 2).saturating_sub(if slopes.len() % 2 == 0 { 1 } else { 0 });
    let median_slope = quickselect(slopes.clone(), median_index, 0, slopes.len().saturating_sub(1));
    let mut small = Vec::new();
    let mut equal = Vec::new();
    let mut large = Vec::new();
    for (i, slope) in slopes.iter().enumerate() {
        if *slope < median_slope {
            small.push(pairs[i]);
        } else if *slope == median_slope {
            equal.push(pairs[i]);
        } else {
            large.push(pairs[i]);
        }
    }
    let max_slope = points
        .iter()
        .map(|point| point.y - median_slope as f32 * point.x)
        .fold(f32::NEG_INFINITY, |a, b| a.max(b));
    let max_set: Vec<Point> = points
        .iter()
        .filter(|&point| point.y - median_slope as f32 * point.x == max_slope)
        .cloned()
        .collect();
    let left = max_set
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap()
        .clone();
    let right = max_set
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap()
        .clone();
    if left.x <= vertical_line && right.x > vertical_line {
        return (left, right);
    } else if right.x <= vertical_line {
        candidates.extend(
            large
                .iter()
                .chain(equal.iter())
                .map(|&(p, _)| p)
                .collect::<Vec<Point>>(),
        );
        candidates.extend(
            small
                .iter()
                .flat_map(|&(p, q)| vec![p, q])
                .collect::<Vec<Point>>(),
        );
    } else {
        candidates.extend(
            small
                .iter()
                .flat_map(|&(p, q)| vec![p, q])
                .collect::<Vec<Point>>(),
        );
        candidates.extend(
            large
                .iter()
                .chain(equal.iter())
                .map(|&(p, _)| p)
                .collect::<Vec<Point>>(),
        );
    }
    bridge(candidates, vertical_line)
}

fn flipped(points: Vec<Point>) -> Vec<Point> {
    points
        .iter()
        .map(|&point| Point::new(-point.x, -point.y))
        .collect()
}

pub fn kirk_patrick_seidel(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    points: Vec<Point>,
    _current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    // let local_drawing_progress = 0;

    // Get upper hull
    let mut points = points;
    points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let lower = *points.first().unwrap();
    let upper = *points.last().unwrap();
    let filtered_points: Vec<Point> = points
        .iter()
        .filter(|&point| (lower.x < point.x) && (point.x < upper.x))
        .cloned()
        .collect();
    let mut upper_hull = connect(lower, upper, filtered_points);

    draw_lines!(
        commands,
        meshes,
        materials,
        upper_hull
            .iter()
            .map(|point| [point.x, point.y, 0.0])
            .collect::<Vec<[f32; 3]>>(),
        ConvexHull
    );

    // Get lower hull
    let mut points = flipped(points);
    points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let lower = *points.first().unwrap();
    let upper = *points.last().unwrap();
    let filtered_points = points
        .iter()
        .filter(|&point| (lower.x < point.x) && (point.x < upper.x))
        .cloned()
        .collect();
    let mut lower_hull = flipped(connect(lower, upper, filtered_points));

    if Some(lower_hull.last()) == Some(upper_hull.first()) {
        upper_hull.pop();
    }

    if Some(lower_hull.first()) == Some(upper_hull.last()) {
        lower_hull.pop();
    }

    let mut convex_hull = upper_hull;
    convex_hull.extend(lower_hull);

    draw_lines!(
        commands,
        meshes,
        materials,
        convex_hull
            .iter()
            .map(|point| [point.x, point.y, 0.0])
            .collect::<Vec<[f32; 3]>>(),
        ConvexHull
    );

    // Update current in loop
    // drawing_in_progress.1 = current;
    // if current == drawing_in_progress.2 {
    drawing_in_progress.0 = false;
    drawing_in_progress.1 = 0;
    // }
}
