use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_pancam::{PanCam, PanCamPlugin};
use rayon::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, PanCamPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .insert_resource(NumberOfPoints::default())
        .insert_resource(PointData::default())
        .insert_resource(Distribution::default())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
        enabled: true,        // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
        min_scale: 1.,        // prevent the camera from zooming too far in
        max_scale: Some(40.), // prevent the camera from zooming too far out
        ..default()
    });
}

#[derive(PartialEq)]
enum DistributionType {
    Fibonacci,
    Random,
}

#[derive(Component)]
struct PointSingle;

#[derive(Component)]
struct ConvexHull;

#[derive(Resource)]
struct NumberOfPoints(usize);

#[derive(Resource, Debug)]
struct PointData(Vec<Vec2>);

#[derive(Resource)]
struct Distribution(DistributionType);

impl Default for NumberOfPoints {
    fn default() -> Self {
        Self(10)
    }
}

impl Default for PointData {
    fn default() -> Self {
        Self(vec![])
    }
}

impl Default for Distribution {
    fn default() -> Self {
        Self(DistributionType::Fibonacci)
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_query: Query<Entity, With<PointSingle>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    mut number_of_points: ResMut<NumberOfPoints>,
    mut point_data: ResMut<PointData>,
    mut distribution: ResMut<Distribution>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=1_000).text("Number of points"));

        if ui.add(egui::Button::new("Generate World")).clicked() {
            for entity in point_query.iter() {
                commands.entity(entity).despawn();
            }
            for entity in convex_hull_query.iter() {
                commands.entity(entity).despawn();
            }
            point_data.0.clear();

            let num_shapes = number_of_points.0;

            let golden_angle = 137.5_f32.to_radians();

            (0..num_shapes).into_iter().for_each(|i| {
                match distribution.0 {
                    DistributionType::Fibonacci => {
                        let index: f32 = (i as f32) - (i as f32) / 2.0;
                        // Distribute colors evenly across the rainbow.
                        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

                        let angle = 2.0 * std::f32::consts::PI * index * (1.0 / golden_angle);
                        let radius = 100.0 * (index - 0.5).sqrt();

                        let x = angle.cos() * radius;
                        let y = angle.sin() * radius;

                        if x.is_nan() || y.is_nan() {
                            return;
                        }

                        point_data.0.push(Vec2::new(x, y));

                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: 10.0 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                    DistributionType::Random => {
                        let radius = number_of_points.0 as f32 * 50.0 * rand::random::<f32>();
                        let angle: f32 = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                        let x = angle.cos() * radius;
                        let y = angle.sin() * radius;

                        // Assuming you have a method to generate a random color for each point
                        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

                        point_data.0.push(Vec2::new(x, y));

                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: 10.0 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                }
            })
        }

        egui::ComboBox::from_label("Select distribution type")
            .selected_text(match distribution.0 {
                DistributionType::Fibonacci => "Fibonacci",
                DistributionType::Random => "Random",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut distribution.0,
                    DistributionType::Fibonacci,
                    "Fibonacci",
                );
                ui.selectable_value(&mut distribution.0, DistributionType::Random, "Random");
            });

        if ui.add(egui::Button::new("Generate Mesh")).clicked() {
            for entity in convex_hull_query.iter() {
                commands.entity(entity).despawn();
            }

            let vertices = jarvis_march(point_data.0.clone());

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(
                        meshes.add(
                            Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices),
                        ),
                    ),
                    material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
                    ..default()
                },
                ConvexHull,
            ));
        }
    });
}

fn jarvis_march(points: Vec<Vec2>) -> Vec<[f32; 3]> {
    let mut hull = vec![];
    let mut start = 0;
    for (i, point) in points.iter().enumerate() {
        if point.x < points[start].x {
            start = i;
        }
    }
    let mut current = start;
    let first_point = [points[current].x as f32, points[current].y as f32, 0.0];
    loop {
        hull.push([points[current].x as f32, points[current].y as f32, 0.0]);
        let mut next = (current + 1) % points.len();
        for (i, point) in points.iter().enumerate() {
            if i != current && i != next {
                let cross = (point.x - points[current].x) * (points[next].y - points[current].y)
                    - (point.y - points[current].y) * (points[next].x - points[current].x);
                if cross < 0.0 {
                    next = i;
                }
            }
        }
        current = next;
        if current == start {
            break;
        }
    }
    hull.push(first_point);
    hull
}
