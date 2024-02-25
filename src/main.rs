use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, PanCamPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .add_systems(Update, drawing_ui)
        .insert_resource(NumberOfPoints(10))
        .insert_resource(PointData(vec![]))
        .insert_resource(Distribution(DistributionType::Fibonacci))
        .insert_resource(DrawingInProgress(false, 0, 0))
        .insert_resource(SimulationTimeSec(1.0))
        .insert_resource(SimulationTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .run();
}

#[derive(Component)]
struct Gizmo;

#[derive(Component)]
struct FinalLine;

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

#[derive(Resource)]
struct SimulationTimeSec(f32);

#[derive(Resource)]
struct DrawingInProgress(bool, usize, usize); // (is_drawing, number of iterations reached, initial state of the drawing)

#[derive(Resource, Debug)]
struct PointData(Vec<Vec2>);

#[derive(Resource)]
struct Distribution(DistributionType);

#[derive(Resource)]
struct SimulationTimer(Timer);

fn drawing_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_query: Query<Entity, With<PointSingle>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    mut number_of_points: ResMut<NumberOfPoints>,
    mut point_data: ResMut<PointData>,
    mut distribution: ResMut<Distribution>,
    mut gizmo_query: Query<Entity, With<Gizmo>>,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
    mut simulation_time: ResMut<SimulationTimeSec>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
) {
    if point_data.0.len() < 1 {
        return;
    }

    if !drawing_in_progress.0 {
        return;
    }

    simulation_timer.0.tick(time.delta());
    if !simulation_timer.0.finished() {
        return;
    }

    let points = point_data.0.clone();

    let mut current = drawing_in_progress.1;

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
        FinalLine,
    ));

    current = next;
    drawing_in_progress.1 = current;
    if current == drawing_in_progress.2 {
        drawing_in_progress.0 = false;
        drawing_in_progress.1 = 0;
        return;
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
    mut gizmo_query: Query<Entity, With<Gizmo>>,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
    mut simulation_time: ResMut<SimulationTimeSec>,
    mut simulation_timer: ResMut<SimulationTimer>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=1_000).text("Number of points"));
        if ui
            .add(egui::Slider::new(&mut simulation_time.0, 0.0..=1.0).text("Simulation time (s)"))
            .changed()
        {
            simulation_timer
                .0
                .set_duration(std::time::Duration::from_secs_f32(simulation_time.0));
        }

        if ui.add(egui::Button::new("Generate World")).clicked() {
            for entity in point_query.iter() {
                commands.entity(entity).despawn();
            }
            if !drawing_in_progress.0 {
                for entity in convex_hull_query.iter() {
                    commands.entity(entity).despawn();
                }
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
            if !drawing_in_progress.0 {
                for entity in convex_hull_query.iter() {
                    commands.entity(entity).despawn();
                }
                drawing_in_progress.0 = true;
                drawing_in_progress.1 = 0;
                let points = point_data.0.clone();
                for (i, point) in points.iter().enumerate() {
                    if point.x < points[drawing_in_progress.1].x {
                        drawing_in_progress.1 = i;
                        drawing_in_progress.2 = i;
                    }
                }
            }
        }
    });
}
