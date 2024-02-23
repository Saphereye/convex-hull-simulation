use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, ui)
        .insert_resource(NumberOfPoints::default())
        .run();
}

const MAX_RADIUS: f32 = 50.0;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct PointSingle(Vec2);

#[derive(Resource)]
struct NumberOfPoints(usize);

impl Default for NumberOfPoints {
    fn default() -> Self {
        Self(10)
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    point_query: Query<Entity, With<PointSingle>>,
    mut number_of_points: ResMut<NumberOfPoints>,
) {
    egui::Window::new("Inspector").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::Slider::new(&mut number_of_points.0, 0..=100).text("Number of points"));

        // TODO Add button to draw convex hull

        if ui.add(egui::Button::new("Generate World")).clicked() {
            for entity in point_query.iter() {
                commands.entity(entity).despawn();
            }

            let num_shapes = number_of_points.0;

            let golden_angle = 137.5_f32.to_radians();

            // TODO Use rayon::par_iter to blit the shapes
            for i in 0..num_shapes {
                let index: f32 = (i as f32) - (i as f32) / 2.0;
                // Distribute colors evenly across the rainbow.
                let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

                let angle = 2.0 * std::f32::consts::PI * index * (1.0 / golden_angle);
                let radius = MAX_RADIUS * (index - 0.5).sqrt();

                let x = angle.cos() * radius;
                let y = angle.sin() * radius;

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Circle { radius: 10.0 })),
                        material: materials.add(color),
                        transform: Transform::from_xyz(x, y, 0.0),
                        ..default()
                    },
                    PointSingle(Vec2::new(x, y)),
                ));
            }
        }
    });
}
