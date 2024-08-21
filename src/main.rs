use std::{collections::HashMap, f32::consts::PI};

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_rapier3d::prelude::*;
use coord::{outline, Coord};
use rand::prelude::*;

mod coord;
mod mesh;

fn main() {
    App::new()
        .add_event::<SpawnHex>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_keypress, move_camera, spawn_hex))
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct MainCameraPhysics;

fn handle_keypress(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    hexes: Res<HexBoard>,
    mut spawn_hex: EventWriter<SpawnHex>,
    mut query_physics: Query<&mut ExternalForce, With<MainCameraPhysics>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
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
    let mut direction = Vec3::ZERO;
    let (x, y) = (Vec3::X, -Vec3::Z);
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= x;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += x;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += y;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= y;
    }
    let direction = direction.normalize_or_zero();
    for mut force in query_physics.iter_mut() {
        force.force = 25.0 * direction;
    }
}

fn move_camera(
    mut query_camera_physics: Query<&Transform, (With<MainCameraPhysics>, Without<MainCamera>)>,
    mut query_camera: Query<&mut Transform, (With<MainCamera>, Without<MainCameraPhysics>)>,
) {
    let Ok(&physics_transform) = query_camera_physics.get_single_mut() else {
        return;
    };
    for mut transform in query_camera.iter_mut() {
        transform.translation = physics_transform.translation;
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
    let camera_position = Transform::from_xyz(0.0, 5.0, 5.0);
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            projection: OrthographicProjection {
                // 15 world units per window height.
                scaling_mode: ScalingMode::FixedVertical(15.0),
                ..default()
            }
            .into(),
            transform: camera_position.looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
    ));
    commands.spawn((
        MainCameraPhysics,
        RigidBody::Dynamic,
        TransformBundle::from(camera_position),
        GravityScale(0.0),
        AdditionalMassProperties::Mass(1.0),
        LockedAxes::ROTATION_LOCKED,
        Sleeping::disabled(),
        ExternalForce::default(),
        Damping {
            linear_damping: 20.0,
            ..Default::default()
        },
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
