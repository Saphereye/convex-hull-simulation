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
//!
//! ## Comparison
//! ### Introduction
//! | Feature        | Kirk Seidel                                                  | Jarvis March                                             |
//! | -------------- | ------------------------------------------------------------ | -------------------------------------------------------- |
//! | Algorithm Type | Divide and Conquer                                           | Incremental/Gift Wrapping                                |
//! | Complexity     | $O(n \log n)$                                                | $O(nh)$                                                  |
//! | Advantages     | Faster for large datasets, Handles non-convex shapes well    | Simplicity of implementation                             |
//! | Disadvantages  | More complex implementation, Potentially higher memory usage | Slower for large datasets, Sensitive to degenerate cases |
//! | Key Features   | Divide and conquer strategy                                  | Iterative selection of points, based on polar angle      |
//! 
//! ### Performance comparison
//! todo explain
//! ![Violin Plot](../violin.svg)
//! 
//! ### Flamegraph
//! ![Flame](../kirk_patrick_flamegraph.svg)

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use bevy_egui::{
    egui::{self},
    systems::InputResources,
    EguiContexts, EguiPlugin,
};

use bevy_pancam::{PanCam, PanCamPlugin};

use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

mod algorithms;
use algorithms::*;

mod distributions;
use distributions::*;

/// Component to identify the points. Used by [despawn_entities] function to despawn all the points.
#[derive(Component)]
struct PointSingle;

/// Resource to contain all data regarding the points.
///
/// It contains data in the following order: The points | text input | point radius | # of points | can add manually
#[derive(Resource)]
struct PointData(Vec<Vec2>, String, f32, usize, bool);

/// The timer for simulation, time step of simulation
#[derive(Resource)]
struct SimulationTimer(Timer, f32);

/// Component to identify the color text.
#[derive(Component)]
struct ColorText;

/// Resource to store the text comment on the screen
#[derive(Resource)]
struct TextComment;

const MAX_ZOOM_OUT: f32 = 500.0;
const TEXT_SIZE: f32 = 30.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, PanCamPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .add_systems(Update, graphics_drawing)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, mouse_position_system)
        .add_systems(Update, check_egui_wants_focus)
        .add_systems(Update, pan_cam_system)
        .insert_resource(PointData(vec![], String::new(), 10.0, 0, false))
        .insert_resource(Distribution(DistributionType::Fibonacci))
        .insert_resource(SimulationTimer(
            Timer::from_seconds(1.0, TimerMode::Repeating),
            1.0,
        ))
        .insert_resource(DrawingHistory(vec![], 0))
        .insert_resource(Algorithm(AlgorithmType::JarvisMarch))
        .insert_resource(TextComment)
        .insert_resource(EguiWantsFocus(false))
        .run();
}

/// Creates a combo box with the given label and choices.
///
/// The combo box is created with the currently selected value. If the current value matches one of the choices,
/// that choice is selected in the combo box. Otherwise, the first choice is selected.
///
/// See [ui] function for usage.
fn create_combo_box<T>(ui: &mut egui::Ui, label: &str, current: &mut T, choices: &[(&str, T)])
where
    T: PartialEq + Copy,
{
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

/// Despawns all entities with the given component.
fn despawn_entities<T: Component>(commands: &mut Commands, query: &Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Initial setup function
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

/// Adds controls for pancam system. Namely disables the camera when egui wants focus.
fn pan_cam_system(egui_wants_focus: Res<EguiWantsFocus>, mut pan_cam: Query<&mut PanCam>) {
    for mut cam in pan_cam.iter_mut() {
        cam.enabled = !egui_wants_focus.0;
    }
}

/// Controls the keyboard input for the simulation.
fn keyboard_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut point_data: ResMut<PointData>,
    egui_resources: InputResources,
) {
    let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    if ctrl && input.just_pressed(KeyCode::KeyD) {
        point_data.1.clear();
    }

    if ctrl && input.just_pressed(KeyCode::KeyV) {
        let clipboard = egui_resources.egui_clipboard;
        match clipboard.get_contents() {
            Some(contents) => {
                point_data.1 += "\n";
                point_data.1 += &contents;
            }
            None => warn!("Clipboard is empty"),
        }
    }
}

/// Draws the graphics as declared in [LineType] enum.
fn graphics_drawing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut simulation_timer: ResMut<SimulationTimer>,
    gizmo_query: Query<Entity, With<Gizmo>>,
    text_query: Query<Entity, With<ColorText>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    mut drawing_history: ResMut<DrawingHistory>,
    window: Query<&mut Window, With<PrimaryWindow>>,
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
                                    vec![[a.x, a.y, 0.0], [b.x, b.y, 0.0]],
                                ),
                            ),
                        ),
                        material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
                        ..default()
                    },
                    ConvexHull,
                ));
            }
            LineType::Temporary(a, b) => {
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
                                    vec![[a.x, a.y, 0.0], [b.x, b.y, 0.0]],
                                ),
                            ),
                        ),
                        material: materials.add(Color::rgb(0.44, 0.44, 0.44)),
                        ..default()
                    },
                    Gizmo,
                ));
            }
            LineType::TextComment(comment) => {
                commands.spawn((
                    TextBundle::from_section(
                        comment,
                        TextStyle {
                            font_size: TEXT_SIZE,
                            ..default()
                        },
                    )
                    .with_text_justify(JustifyText::Center)
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(5.0),
                        left: Val::Px(5.0),
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

/// Resource to store whether egui wants focus or not.
#[derive(Resource, PartialEq)]
struct EguiWantsFocus(bool);

/// Checks if egui wants focus or not by checking if the pointer is over the egui area.
fn check_egui_wants_focus(
    mut contexts: Query<&mut bevy_egui::EguiContext>,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let ctx = contexts.iter_mut().next();
    let new_wants_focus = if let Some(ctx) = ctx {
        let ctx = ctx.into_inner().get_mut();
        ctx.is_pointer_over_area()
    } else {
        false
    };
    wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
}

/// System to add points to the world by clicking.
fn mouse_position_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut point_data: ResMut<PointData>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera>>,
    egui_wants_focus: Res<EguiWantsFocus>,
) {
    if egui_wants_focus.0 {
        return;
    }

    if !point_data.4 {
        return;
    }

    let window = window.single_mut();
    let (camera_transform, camera) = camera_query.single();

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let world_position = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
            .unwrap();

        point_data
            .0
            .push(Vec2::new(world_position.x, world_position.y));
        point_data.3 += 1;

        let color = Color::WHITE;

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle {
                    radius: point_data.2,
                })),
                material: materials.add(color),
                transform: Transform::from_xyz(world_position.x, world_position.y, 0.0),
                ..default()
            },
            PointSingle,
        ));
    }
}

/// Draws the UI for the simulation.
fn ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut point_data: ResMut<PointData>,
    mut distribution: ResMut<Distribution>,
    mut simulation_timer: ResMut<SimulationTimer>,
    mut algorithm: ResMut<Algorithm>,
    mut drawing_history: ResMut<DrawingHistory>,
    point_query: Query<Entity, With<PointSingle>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    gizmo_query: Query<Entity, With<Gizmo>>,
    text_query: Query<Entity, With<ColorText>>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.label("Choose the number of points and the simulation time Δt.");
        ui.add(egui::Slider::new(&mut point_data.3, 0..=15_000).text("Number of points"));
        if ui
            .add(egui::Slider::new(&mut simulation_timer.1, 0.0..=10.0).text("Simulation time (s)"))
            .changed()
        {
            let simulation_timer_time = simulation_timer.1;
            simulation_timer
                .0
                .set_duration(std::time::Duration::from_secs_f32(simulation_timer_time));

        }

        ui.add(egui::Slider::new(&mut point_data.2, 1.00..=1000.0).text("Point radius"));

        ui.separator();

        ui.label("Select the distribution type and click `Generate world` to generate the points based on that");

        create_combo_box(
            ui,
            "Select distribution type",
            &mut distribution.0,
            &[
                ("Fibonacci", DistributionType::Fibonacci),
                ("Circle Perimeter", DistributionType::CirclePerimeter),
                ("Square Perimeter", DistributionType::Square),
                ("Random", DistributionType::Random),
            ],
        );

        if ui.button("Clear world").clicked() {
            despawn_entities(&mut commands, &point_query);
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
            despawn_entities(&mut commands, &text_query);
            point_data.0.clear();
            drawing_history.0.clear();
        }

        ui.checkbox(&mut point_data.4, "Manually add points by clicking");

        // ui.text_edit_multiline(&mut point_data.1);
        ui.code_editor(&mut point_data.1);

        if ui.button("Generate World").clicked() {
            despawn_entities(&mut commands, &point_query);
            despawn_entities(&mut commands, &convex_hull_query);
            despawn_entities(&mut commands, &gizmo_query);
            despawn_entities(&mut commands, &text_query);
            point_data.0.clear();
            drawing_history.0.clear();

            if point_data.1.is_empty() {
                (0..=point_data.3).for_each(|i| match distribution.0 {
                    DistributionType::Fibonacci => {
                        let color = Color::hsl(360. * i as f32 / point_data.3 as f32, 0.95, 0.7);
                        let (x, y) = fibonacci_circle(i);
                        if x.is_nan() || y.is_nan() {
                            return;
                        }
                        point_data.0.push(Vec2::new(x, y));
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_data.2 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                    DistributionType::Random => {
                        let (x, y) = bounded_random(point_data.3);
                        let color = Color::hsl(360. * i as f32 / point_data.3 as f32, 0.95, 0.7);
                        point_data.0.push(Vec2::new(x, y));
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_data.2 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                    DistributionType::CirclePerimeter => {
                        let (x, y) = circle_points(point_data.3);
                        let color = Color::hsl(360. * i as f32 / point_data.3 as f32, 0.95, 0.7);
                        point_data.0.push(Vec2::new(x, y));
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_data.2 })),
                                material: materials.add(color),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            PointSingle,
                        ));
                    }
                    DistributionType::Square => {
                        let (x, y) = bounded_random_square(point_data.3);
                        let color = Color::hsl(360. * i as f32 / point_data.3 as f32, 0.95, 0.7);
                        point_data.0.push(Vec2::new(x, y));
                        commands.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Circle { radius: point_data.2 })),
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
                                    mesh: Mesh2dHandle(meshes.add(Circle { radius: point_data.2 })),
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
    });
}
