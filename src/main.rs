use bevy::window::PrimaryWindow;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Node {
    x_position: f32,
    y_position: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, mouse_click_system)
        .run();
}

fn mouse_click_system(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        info!("World coords: {}/{}", world_position.x, world_position.y);

        if mouse_button_input.just_pressed(MouseButton::Left) {
            info!("left mouse just pressed");
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: bevy::sprite::Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
                    material: materials.add(Color::hsl(1.0, 0.95, 0.7)),
                    transform: Transform::from_xyz(world_position.x as f32, world_position.y as f32, 0.0),
                    ..default()
                },
                Node {
                    x_position: world_position.x as f32,
                    y_position: world_position.y as f32,
                },
            ));
        }
    }
}

const X_EXTENT: f32 = 600.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
