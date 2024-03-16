//! # Convex Hull Simulation
//! A rust based step by step simulation of Jarvis March and Kirk Patrick Seidel algorithms for convex hull generation.
//! The program uses [Bevy](https://bevyengine.org) as the game engine and [egui](https://github.com/emilk/egui) for the ui library.
//!
//! ## Program flow
//! [![](https://mermaid.ink/img/pako:eNpVkMFuwjAMhl_FyolK8AI9TFqZNmkb0gTXXkxiiNUkRm7CNlHefWHdDvhk2f5-2__FWHFkWnMI8mk9aob3bZ-gxuNi7UVGgpNwyuB4zMr7kllSA6vVw_TMe0loLU_Q_c9aSWf6Al9CAAxHUc4-Nn-CNwqmTkpy5GCLyUms7Nzt5u4r6plH2KBaP8F68UKJFPOdcnNHvLEO8IH1ODvAjthRqKBZmkgakV197nIDepM9RepNW1OHOvSmT9c6hyXL7jtZ02YttDTl5OrCJ8ajYjTtAcNYq-Q4i25mt35Nu_4AUKJpKA?type=png)](https://mermaid.live/edit#pako:eNpVkMFuwjAMhl_FyolK8AI9TFqZNmkb0gTXXkxiiNUkRm7CNlHefWHdDvhk2f5-2__FWHFkWnMI8mk9aob3bZ-gxuNi7UVGgpNwyuB4zMr7kllSA6vVw_TMe0loLU_Q_c9aSWf6Al9CAAxHUc4-Nn-CNwqmTkpy5GCLyUms7Nzt5u4r6plH2KBaP8F68UKJFPOdcnNHvLEO8IH1ODvAjthRqKBZmkgakV197nIDepM9RepNW1OHOvSmT9c6hyXL7jtZ02YttDTl5OrCJ8ajYjTtAcNYq-Q4i25mt35Nu_4AUKJpKA)
//!
//! ## Comparison between Jarvis March and Kirk Patrick Seidel
//! todo

use std::fmt::{Debug, Display};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_pancam::{PanCam, PanCamPlugin};

mod algorithms;
use algorithms::*;

mod distributions;
use distributions::*;

#[derive(Component)]
struct PointSingle;

#[derive(Resource)]
struct NumberOfPoints(usize);

#[derive(Resource)]
struct SimulationTimeSec(f32);

#[derive(Resource, Debug)]
struct PointData(Vec<Vec2>);

#[derive(Resource)]
struct SimulationTimer(Timer);

fn create_combo_box<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    label: &str,
    current: &mut T,
    choices: &[(&str, T)],
) {
    egui::ComboBox::from_label(label)
        .selected_text(choices.iter().find(|(_, value)| *value == *current).unwrap().0)
        .show_ui(ui, |ui| {
            for (name, value) in choices {
                ui.selectable_value(current, *value, *name);
            }
        });
}

#[inline]
fn despawn_entities<T: Component>(commands: &mut Commands, query: &Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, PanCamPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .add_systems(Update, graphics_drawing)
        .insert_resource(NumberOfPoints(10))
        .insert_resource(PointData(vec![]))
        .insert_resource(Distribution(DistributionType::Fibonacci))
        .insert_resource(DrawingInProgress(false, 0, 0))
        .insert_resource(SimulationTimeSec(1.0))
        .insert_resource(SimulationTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(Algorithm(AlgorithmType::JarvisMarch))
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

fn graphics_drawing(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    point_data: ResMut<PointData>,
    drawing_in_progress: ResMut<DrawingInProgress>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
    algorithm: ResMut<Algorithm>,
    gizmo_query: Query<Entity, With<Gizmo>>,
) {
    if point_data.0.len() < 1 || !drawing_in_progress.0 {
        despawn_entities(&mut commands, &gizmo_query);
        return;
    }

    simulation_timer.0.tick(time.delta());
    if !simulation_timer.0.finished() {
        return;
    }
    despawn_entities(&mut commands, &gizmo_query);

    match algorithm.0 {
        AlgorithmType::JarvisMarch => {
            jarvis_march(
                &mut commands,
                meshes,
                materials,
                point_data.0.clone(),
                drawing_in_progress.1,
                drawing_in_progress,
            );
        }
        AlgorithmType::KirkPatrickSeidel => {
            kirk_patrick_seidel(
                &mut commands,
                meshes,
                materials,
                point_data.0.clone(),
                drawing_in_progress.1,
                drawing_in_progress,
            );
        }
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
    gizmo_query: Query<Entity, With<Gizmo>>,
    mut drawing_in_progress: ResMut<DrawingInProgress>,
    mut simulation_time: ResMut<SimulationTimeSec>,
    mut simulation_timer: ResMut<SimulationTimer>,
    mut algorithm: ResMut<Algorithm>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=100_000).text("Number of points"));
        if ui
            .add(egui::Slider::new(&mut simulation_time.0, 0.0..=1.0).text("Simulation time (s)"))
            .changed()
        {
            simulation_timer
                .0
                .set_duration(std::time::Duration::from_secs_f32(simulation_time.0));
        }

        if ui.add(egui::Button::new("Generate World")).clicked() {
            despawn_entities(&mut commands, &point_query);
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
            drawing_in_progress.0 = false;
            drawing_in_progress.1 = 0;
            drawing_in_progress.2 = 0;
            point_data.0.clear();

            (0..number_of_points.0)
                .into_iter()
                .for_each(|i| match distribution.0 {
                    DistributionType::Fibonacci => {
                        let color =
                            Color::hsl(360. * i as f32 / number_of_points.0 as f32, 0.95, 0.7);
                        let (x, y) = fibonacci_circle(i);
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
                        let (x, y) = bounded_random(number_of_points.0);
                        let color =
                            Color::hsl(360. * i as f32 / number_of_points.0 as f32, 0.95, 0.7);
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
                })
        }

        create_combo_box(
            ui,
            "Select distribution type",
            &mut distribution.0,
            &[
                ("Fibonacci", DistributionType::Fibonacci),
                ("Random", DistributionType::Random),
            ],
        );

        create_combo_box(
            ui,
            "Select Algorithm Type",
            &mut algorithm.0,
            &[
                ("Jarvis March", AlgorithmType::JarvisMarch),
                ("Kirk Patrick Seidel", AlgorithmType::KirkPatrickSeidel),
            ],
        );

        if ui.add(egui::Button::new("Generate Mesh")).clicked() {
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
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
    });
}
