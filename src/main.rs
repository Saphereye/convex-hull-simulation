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
        .run();
}

const MAX_RADIUS: f32 = 50.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    let mut shapes = vec![];
    for _i in 0..100 {
        shapes.push(Mesh2dHandle(meshes.add(Circle { radius: 10.0 })));
    }
    let num_shapes = shapes.len();

    let golden_angle = 137.5_f32.to_radians();

    for (i, shape) in shapes.into_iter().enumerate() {
        let index: f32 = (i as f32) - (i as f32)/2.0;
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        let angle = 2.0 * std::f32::consts::PI * index * (1.0 / golden_angle);
        let radius = MAX_RADIUS * (index - 0.5).sqrt();

        let x = angle.cos() * radius;
        let y = angle.sin() * radius;

        commands.spawn(MaterialMesh2dBundle {
            mesh: shape,
            material: materials.add(color),
            transform: Transform::from_xyz(x, y, 0.0),
            ..default()
        });
    }
}

fn ui(mut contexts: EguiContexts) {
    egui::SidePanel::left("side_panel").show(contexts.ctx_mut(), |ui| {
        ui.label("side panel");
    });
}
