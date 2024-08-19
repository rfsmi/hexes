use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_rapier3d::prelude::*;

mod mesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_keypress, move_camera))
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct MainCameraPhysics;

fn handle_keypress(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query_physics: Query<&mut ExternalForce, With<MainCameraPhysics>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
    let mut direction = Vec3::ZERO;
    let (x, y) = (Vec3::X, -Vec3::Z);
    // let (x, y) = (Vec3::X - Vec3::Z, -Vec3::X - Vec3::Z);
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

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    let camera_position = Transform::from_xyz(0.0, 5.0, 5.0);
    commands.spawn((
        MainCamera,
        Camera3dBundle {
            projection: OrthographicProjection {
                // 6 world units per window height.
                scaling_mode: ScalingMode::FixedVertical(6.0),
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

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });

    let hex: Handle<Mesh> = meshes.add(mesh::generate(6));
    // hexes
    commands.spawn(PbrBundle {
        mesh: hex.clone(),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
        transform: Transform::from_xyz(2.5, 0.5, 0.5),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: hex.clone(),
        material: materials.add(Color::rgb(0.0, 1.0, 0.0)),
        transform: Transform::from_xyz(0.5, 2.5, 0.5).with_scale([1.0, 2.0, 1.0].into()),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: hex,
        material: materials.add(Color::rgb(0.0, 0.0, 1.0)),
        transform: Transform::from_xyz(0.5, 0.5, 2.5),
        ..default()
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
