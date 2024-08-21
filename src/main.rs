use std::{collections::HashMap, f32::consts::PI};

use bevy::{prelude::*, render::camera::ScalingMode};
use coord::{outline, Coord};
use rand::prelude::*;

mod coord;
mod mesh;

fn main() {
    App::new()
        .add_event::<SpawnHex>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                exit_on_esc,
                spawn_hex_on_space,
                move_camera_on_wasd,
                spawn_hex,
            ),
        )
        .run();
}

fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn spawn_hex_on_space(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    hexes: Res<HexBoard>,
    mut spawn_hex: EventWriter<SpawnHex>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Spawn a hex
        let coord = if hexes.board.is_empty() {
            Coord::new(0, 0)
        } else {
            let hexes = hexes.board.keys().copied();
            *outline(hexes).choose(&mut thread_rng()).unwrap()
        };
        let colour = Color::hsv(
            thread_rng().gen_range(0.0..=360.0),
            thread_rng().gen_range(0.6..=0.9),
            thread_rng().gen_range(0.7..=1.0),
        );
        spawn_hex.send(SpawnHex(coord, colour));
    }
}

#[derive(Component)]
struct CameraSpeed(f32);

fn move_camera_on_wasd(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &CameraSpeed), With<Camera>>,
    mut hold_duration: Local<f32>,
) {
    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction -= Vec3::Z;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }
    direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        *hold_duration = 0.0;
    } else {
        *hold_duration += time.delta_seconds();
    }
    // Ease movement out over 0.5 seconds
    let factor = 1.0 - (1.0 - *hold_duration / 0.5).powi(5);
    direction *= time.delta_seconds() * factor.clamp(0.0, 1.0);
    for (mut transform, speed) in camera_query.iter_mut() {
        transform.translation += direction * speed.0;
    }
}

#[derive(Event)]
struct SpawnHex(Coord, Color);

#[derive(Resource)]
struct HexBoard {
    mesh: Handle<Mesh>,
    board: HashMap<Coord, Entity>,
}

fn spawn_hex(
    mut events: EventReader<SpawnHex>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hexes: ResMut<HexBoard>,
) {
    for SpawnHex(coord, colour) in events.read() {
        if let Some(hex) = hexes.board.get(coord) {
            commands.entity(*hex).despawn();
        }
        let hex = commands.spawn(PbrBundle {
            transform: Transform::default()
                .with_rotation(Quat::from_rotation_y(2.0 * PI / 12.0))
                .with_translation(coord.on_y_plane(0.0)),
            material: materials.add(*colour),
            mesh: hexes.mesh.clone(),
            ..default()
        });
        hexes.board.insert(*coord, hex.id());
    }
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // camera
    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                // 15 world units per window height.
                scaling_mode: ScalingMode::FixedVertical(15.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraSpeed(15.0),
    ));

    // hexes
    commands.insert_resource(HexBoard {
        mesh: meshes.add(mesh::generate(6).scaled_by(Vec3::new(1.0, 0.05, 1.0))),
        board: HashMap::new(),
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });
}
