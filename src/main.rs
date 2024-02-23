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

#[derive(Component)]
struct PointSingle;

#[derive(Component)]
struct ConvexHull;

#[derive(Resource)]
struct NumberOfPoints(usize);

#[derive(Resource)]
struct PointData(Vec<Vec2>);

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

fn ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_query: Query<Entity, With<PointSingle>>,
    convex_hull_query: Query<Entity, With<ConvexHull>>,
    mut number_of_points: ResMut<NumberOfPoints>,
    mut point_data: ResMut<PointData>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=1_000).text("Number of points"));

        if ui.add(egui::Button::new("Generate World")).clicked() {
            for entity in point_query.iter() {
                commands.entity(entity).despawn();
            }
            point_data.0.clear();

            let num_shapes = number_of_points.0;

            let golden_angle = 137.5_f32.to_radians();

            (0..num_shapes).into_iter().for_each(|i| {
                let index: f32 = (i as f32) - (i as f32) / 2.0;
                // Distribute colors evenly across the rainbow.
                let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

                let angle = 2.0 * std::f32::consts::PI * index * (1.0 / golden_angle);
                let radius = 100.0 * (index - 0.5).sqrt();

                let x = angle.cos() * radius;
                let y = angle.sin() * radius;

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
            })
        }

        if ui.add(egui::Button::new("Generate Mesh")).clicked() {
            for entity in convex_hull_query.iter() {
                commands.entity(entity).despawn();
            }
            let vertices: Vec<[f32; 3]> = point_data
                .0
                .iter()
                .map(|point| [point.x as f32, point.y as f32, 0.0])
                .collect();

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
