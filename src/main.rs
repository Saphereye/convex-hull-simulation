//! # Convex Hull Simulation
//! A rust based step by step simulation of Jarvis March and Kirk Patrick Seidel algorithms for convex hull generation.
//! The program uses [Bevy](https://bevyengine.org) as the game engine and [egui](https://github.com/emilk/egui) for the ui library.
//!
//! ## What is a convex hull?
//! The convex hull of a finite point set S in the plane is the smallest
//! convex polygon containing the set. The vertices (corners) of this polygon must be
//! points of S. Thus in order to compute the convex hull of a set S it is necessary to find
//! those points of S which are vertices of the hull. For the purposes of constructing upper
//! bounds we define the convex hull problem, as the problem of constructing the ordered
//! sequence of points of S which constitute the sequences of vertices around the hull.
//!
//! ## Program flow
//! [![](https://mermaid.ink/img/pako:eNpVkMFuwjAMhl_FyolK8AI9TFqZNmkb0gTXXkxiiNUkRm7CNlHefWHdDvhk2f5-2__FWHFkWnMI8mk9aob3bZ-gxuNi7UVGgpNwyuB4zMr7kllSA6vVw_TMe0loLU_Q_c9aSWf6Al9CAAxHUc4-Nn-CNwqmTkpy5GCLyUms7Nzt5u4r6plH2KBaP8F68UKJFPOdcnNHvLEO8IH1ODvAjthRqKBZmkgakV197nIDepM9RepNW1OHOvSmT9c6hyXL7jtZ02YttDTl5OrCJ8ajYjTtAcNYq-Q4i25mt35Nu_4AUKJpKA?type=png)](https://mermaid.live/edit#pako:eNpVkMFuwjAMhl_FyolK8AI9TFqZNmkb0gTXXkxiiNUkRm7CNlHefWHdDvhk2f5-2__FWHFkWnMI8mk9aob3bZ-gxuNi7UVGgpNwyuB4zMr7kllSA6vVw_TMe0loLU_Q_c9aSWf6Al9CAAxHUc4-Nn-CNwqmTkpy5GCLyUms7Nzt5u4r6plH2KBaP8F68UKJFPOdcnNHvLEO8IH1ODvAjthRqKBZmkgakV197nIDepM9RepNW1OHOvSmT9c6hyXL7jtZ02YttDTl5OrCJ8ajYjTtAcNYq-Q4i25mt35Nu_4AUKJpKA)

use std::fmt::Debug;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use egui_extras::{Column, TableBuilder};

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
struct PointData(Vec<Vec2>, String);

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct PointRadius(f32);

fn create_combo_box<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    label: &str,
    current: &mut T,
    choices: &[(&str, T)],
) {
    egui::ComboBox::from_label(label)
        .selected_text(
            choices
                .iter()
                .find(|(_, value)| *value == *current)
                .unwrap()
                .0,
        )
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
        .add_plugins((DefaultPlugins, EguiPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .add_systems(Update, graphics_drawing)
        .insert_resource(NumberOfPoints(0))
        .insert_resource(PointData(vec![], String::new()))
        .insert_resource(Distribution(DistributionType::Fibonacci))
        .insert_resource(SimulationTimeSec(1.0))
        .insert_resource(SimulationTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(DrawingHistory(vec![], 0))
        .insert_resource(Algorithm(AlgorithmType::JarvisMarch))
        .insert_resource(TextComment)
        .insert_resource(PointRadius(10.0))
        .run();
}

#[derive(Component)]
struct ColorText;

#[derive(Resource)]
struct TextComment;

const MAX_ZOOM_OUT: f32 = 500.0;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
        enabled: true,        // when false, controls are disabled. See toggle example.
        zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
        min_scale: 1.,        // prevent the camera from zooming too far in
        max_scale: Some(MAX_ZOOM_OUT), // prevent the camera from zooming too far out
        ..default()
    });
}

use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

macro_rules! draw_lines {
    ($commands:expr, $meshes:expr, $materials:expr, $line:expr, $line_type:tt) => {
        let color = match stringify!($line_type) {
            "Gizmo" => Color::rgb(0.44, 0.44, 0.44),
            "ConvexHull" => Color::rgb(1.0, 1.0, 1.0),
            _ => Color::rgb(0.5, 0.5, 0.5), // Default color
        };

        $commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(
                    $meshes.add(
                        Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default())
                            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, $line),
                    ),
                ),
                material: $materials.add(color),
                ..default()
            },
            $line_type,
        ));
    };
}

fn graphics_drawing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut standard_material: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
    gizmo_query: Query<Entity, With<Gizmo>>,
    text_query: Query<Entity, With<ColorText>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    mut drawing_history: ResMut<DrawingHistory>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    if drawing_history.0.is_empty() || drawing_history.0.len() == drawing_history.1 {
        // despawn_entities(&mut commands, &gizmo_query);
        return;
    }

    simulation_timer.0.tick(time.delta());
    if !simulation_timer.0.finished() {
        return;
    }

    despawn_entities(&mut commands, &gizmo_query);
    despawn_entities(&mut commands, &text_query);

    for i in &drawing_history.0[drawing_history.1] {
        match i {
            LineType::PartOfHull(a, b) => {
                draw_lines!(
                    commands,
                    meshes,
                    materials,
                    vec![[a.x, a.y, 0.0], [b.x, b.y, 0.0]],
                    ConvexHull
                );
            }
            LineType::Temporary(a, b) => {
                draw_lines!(
                    commands,
                    meshes,
                    materials,
                    vec![[a.x, a.y, 0.0], [b.x, b.y, 0.0]],
                    Gizmo
                );
            }
            LineType::TextComment(comment) => {
                commands.spawn((
                    TextBundle::from_section(
                        comment,
                        TextStyle {
                            font_size: 20.0,
                            ..default()
                        },
                    )
                    .with_text_justify(JustifyText::Center)
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(5.0),
                        right: Val::Px(5.0),
                        ..default()
                    }),
                    ColorText,
                ));
            }
            LineType::VerticalLine(x) => {
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(
                            meshes.add(
                                Mesh::new(
                                    PrimitiveTopology::LineStrip,
                                    RenderAssetUsages::default(),
                                )
                                .with_inserted_attribute(
                                    Mesh::ATTRIBUTE_POSITION,
                                    vec![
                                        [*x, -window.height() * MAX_ZOOM_OUT, 0.0],
                                        [*x, window.height() * MAX_ZOOM_OUT, 0.0],
                                    ],
                                ),
                            ),
                        ),
                        material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
                        ..default()
                    },
                    Gizmo,
                ));
            }
            LineType::ClearScreen => {
                despawn_entities(&mut commands, &gizmo_query);
                despawn_entities(&mut commands, &text_query);
                despawn_entities(&mut commands, &convex_hull_query);
            }
        }
    }

    drawing_history.1 += 1;
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
    mut simulation_time: ResMut<SimulationTimeSec>,
    mut simulation_timer: ResMut<SimulationTimer>,
    mut algorithm: ResMut<Algorithm>,
    mut drawing_history: ResMut<DrawingHistory>,
    text_query: Query<Entity, With<ColorText>>,
    mut point_radius: ResMut<PointRadius>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.label("Choose the number of points and the simulation time Δt.");
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=1_00_000).text("Number of points"));
        if ui
            .add(egui::Slider::new(&mut simulation_time.0, 0.0..=10.0).text("Simulation time (s)"))
            .changed()
        {
            simulation_timer
                .0
                .set_duration(std::time::Duration::from_secs_f32(simulation_time.0));

        }
        
        ui.add(egui::Slider::new(&mut point_radius.0, 1.00..=100.0).text("Point radius"));

        ui.separator();

        ui.label("Select the distribution type and click `Generate world` to generate the points based on that");

        create_combo_box(
            ui,
            "Select distribution type",
            &mut distribution.0,
            &[
                ("Fibonacci", DistributionType::Fibonacci),
                ("Random", DistributionType::Random),
            ],
        );

        ui.text_edit_multiline(&mut point_data.1);

        if ui.button("Generate World").clicked() {
            despawn_entities(&mut commands, &point_query);
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
            despawn_entities(&mut commands, &text_query);
            point_data.0.clear();
            drawing_history.0.clear();

            if point_data.1.is_empty() {
                (0..=number_of_points.0).for_each(|i| match distribution.0 {
                    DistributionType::Fibonacci => {
                        let color = Color::hsl(360. * i as f32 / number_of_points.0 as f32, 0.95, 0.7);
                        let (x, y) = fibonacci_circle(i);
                        if x.is_nan() || y.is_nan() {
                            return;
                        }
    
                        point_data.0.push(Vec2::new(x, y));
    
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_radius.0 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                    DistributionType::Random => {
                        let (x, y) = bounded_random(number_of_points.0);
                        let color = Color::hsl(360. * i as f32 / number_of_points.0 as f32, 0.95, 0.7);
                        point_data.0.push(Vec2::new(x, y));
    
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_radius.0 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                })
            } else {
                let lines_copy = point_data.1.clone();
                for (index, line) in lines_copy.lines().enumerate() {
                    let mut split = line.split(',');
                    let x = split.next().and_then(|s| s.trim().parse::<f32>().ok());
                    let y = split.next().and_then(|s| s.trim().parse::<f32>().ok());
                    let color = Color::hsl(360. * index as f32 / point_data.1.len() as f32, 0.95, 0.7);

                    match (x, y) {
                        (Some(x), Some(y)) => {
                            point_data.0.push(Vec2::new(x, y));

                            commands.spawn((
                                MaterialMesh2dBundle {
                                    mesh: Mesh2dHandle(meshes.add(Circle { radius: point_radius.0 })),
                                    material: materials.add(color),
                                    transform: Transform::from_xyz(x, y, 0.0),
                                    ..default()
                                },
                                PointSingle,
                            ));
                        }
                        _ => {
                            eprintln!("Failed to parse line: {}, x: {:?}, y: {:?}", line, x, y);
                        }
                    }
                }
            }
            
        }

        ui.separator();

        ui.label("Select the algorithm type and click `Generate Mesh` to generate the convex hull based on the points");

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
            drawing_history.1 = 0;
            drawing_history.0.clear();
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
            let points = point_data.0.clone();
            match algorithm.0 {
                AlgorithmType::JarvisMarch => jarvis_march(points, &mut drawing_history.0),
                AlgorithmType::KirkPatrickSeidel => kirk_patrick_seidel(points, &mut drawing_history.0),
            };
        }

        ui.separator();
        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("ID");
                });
            })
            .body(|mut body| {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Adarsh Das");
                    });
                    row.col(|ui| {
                        ui.label("2021A7PS1511H");
                    });
                });

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Divyateja Pasupuleti");
                    });
                    row.col(|ui| {
                        ui.label("2021A7PS0075H");
                    });
                });

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Manan Gupta");
                    });
                    row.col(|ui| {
                        ui.label("2021A7PS2091H");
                    });
                });

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Kumarasamy Chelliah");
                    });
                    row.col(|ui| {
                        ui.label("2021A7PS0096H");
                    });
                });

            });
    });
}
