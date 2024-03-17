use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Resource)]
pub struct DrawingInProgress(pub bool, pub usize, pub usize); // (is_drawing, number of iterations reached, initial state of the drawing)

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
    points: Vec<Vec2>,
    current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    let mut current = current;
    let previous_point = [points[current].x, points[current].y, 0.0];
    let mut next = (current + 1) % points.len();
    for (i, point) in points.iter().enumerate() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(
                    meshes.add(
                        Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_POSITION,
                                vec![
                                    [previous_point[0], previous_point[1], 0.0],
                                    [point.x, point.y, 0.0],
                                ],
                            ),
                    ),
                ),
                material: materials.add(Color::rgb(0.44, 0.44, 0.44)),
                ..default()
            },
            Gizmo,
        ));
        if i != current && i != next {
            let cross = (point.x - points[current].x) * (points[next].y - points[current].y)
                - (point.y - points[current].y) * (points[next].x - points[current].x);
            if cross < 0.0 {
                next = i;
            }
        }
    }

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(
                meshes.add(
                    Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![
                                [previous_point[0], previous_point[1], 0.0],
                                [points[next].x, points[next].y, 0.0],
                            ],
                        ),
                ),
            ),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
            ..default()
        },
        ConvexHull,
    ));

    current = next;
    drawing_in_progress.1 = current;
    if current == drawing_in_progress.2 {
        drawing_in_progress.0 = false;
        drawing_in_progress.1 = 0;
    }
}

fn connect(lower: Vec2, upper: Vec2, points: Vec<Vec2>) -> Vec<Vec2> {
    todo!("Implement connect")
}

fn flipped(points: Vec<Vec2>) -> Vec<Vec2> {
    points
        .iter()
        .map(|&point| Vec2::new(-point.x, -point.y))
        .collect()
}

pub fn kirk_patrick_seidel(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    points: Vec<Vec2>,
    current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    let local_drawing_progress = 0;

    // Get upper hull
    let mut points = points;
    points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let lower = *points.first().unwrap();
    let upper = *points.last().unwrap();
    let filtered_points = points
        .iter()
        .filter(|&point| (lower.x < point.x) && (point.x < upper.x))
        .cloned()
        .collect();
    let upper_hull = connect(lower, upper, filtered_points);

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
    let lower_hull = flipped(connect(lower, upper, filtered_points));

    let mut convex_hull = upper.clone();

    // Update current in loop
    drawing_in_progress.1 = current;
    if current == drawing_in_progress.2 {
        drawing_in_progress.0 = false;
        drawing_in_progress.1 = 0;
    }
}
