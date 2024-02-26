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

#[derive(PartialEq)]
pub enum AlgorithmType {
    JarvisMarch,
    KirkPatrickSeidel,
}

#[derive(Resource)]
pub struct Algorithm(pub AlgorithmType);

pub fn jarvis_march(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    points: Vec<Vec2>,
    current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    let mut current = current;
    let previous_point = [points[current].x as f32, points[current].y as f32, 0.0];
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
                                    [point.x as f32, point.y as f32, 0.0],
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

    info!(
        "Drawing final line from {:?} to {:?}",
        previous_point, points[next]
    );
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(
                meshes.add(
                    Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            vec![
                                [previous_point[0], previous_point[1], 0.0],
                                [points[next].x as f32, points[next].y as f32, 0.0],
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
        return;
    }
}

pub fn kirk_patrick_seidel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    points: Vec<Vec2>,
    current: usize,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
) {
    todo!("Implement Kirk-Patrick Seidel algorithm")
}