mod low_res;
mod movement;
mod toon_material;
mod utils;

use bevy::{
  core_pipeline::{
    clear_color::ClearColorConfig,
    prepass::{DepthPrepass, NormalPrepass},
  },
  prelude::*,
  render::camera::ScalingMode,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

use crate::{
  low_res::{LowResCamera, LowResPlugin},
  movement::{apply_user_movement, MaxSpeeds, UserMovement},
  toon_material::ToonMaterial,
};

fn spawn_player(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ToonMaterial>>,
) {
  let main_material_handle =
    materials.add(ToonMaterial::from(Color::rgb(1.0, 1.0, 1.0)));
  // materials.add(ToonMaterial::default());

  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(Mesh::try_from(shape::Cube::new(1.0)).unwrap()),
      material: main_material_handle.clone(),
      transform: Transform::from_xyz(1.5, 0.0, 0.0),
      ..default()
    },
    MaxSpeeds::default(),
    UserMovement,
    Collider::cuboid(0.5, 0.5, 0.5),
    RigidBody::KinematicPositionBased,
    KinematicCharacterController::default(),
  ));
}

fn spawn_props(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn a plane below the player
  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(
        Mesh::try_from(shape::Plane {
          size:         10.0,
          subdivisions: 10,
        })
        .unwrap(),
      ),
      material: materials.add(ToonMaterial::from(Color::rgb(0.2, 0.2, 0.2))),
      transform: Transform::from_xyz(0.0, -0.5, 0.0),
      ..default()
    },
    Collider::cuboid(5.0, 0.05, 5.0),
  ));

  // spawn a capsule near the player
  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(
        Mesh::try_from(shape::Capsule {
          radius: 0.5,
          depth: 1.0,
          ..default()
        })
        .unwrap(),
      ),
      material: materials.add(ToonMaterial {
        color: Color::rgb(0.6, 0.2, 0.2),
        specular_power: 16.0,
        rim_power: 0.5,
        outline_scale: 0.5,
        ..default()
      }),
      transform: Transform::from_xyz(0.0, 1.0, 0.0),
      ..default()
    },
    Collider::capsule_y(0.5, 0.5),
    RigidBody::Dynamic,
  ));
}

fn spawn_camera_and_lights(mut commands: Commands) {
  commands.spawn((
    Camera3dBundle {
      // set the clear color to black
      camera_3d: Camera3d {
        clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
        ..default()
      },
      // position the camera
      transform: Transform::from_xyz(8.0, 8.0 / 2.0_f32.sqrt(), 8.0)
        .looking_at(Vec3::default(), Vec3::Y),
      // use an orthographic projection
      projection: OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize(50.0),
        ..default()
      }
      .into(),
      ..default()
    },
    DepthPrepass,
    NormalPrepass,
    LowResCamera {
      pixel_size: 4,
      ..default()
    },
  ));

  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    // graphics
    .add_plugins(MaterialPlugin::<ToonMaterial>::default())
    .add_plugins(LowResPlugin)
    .insert_resource(Msaa::Off)
    // physics
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugins(RapierDebugRenderPlugin::default())
    // inspector
    .add_plugins(WorldInspectorPlugin::new())
    .register_type::<LowResCamera>()
    .register_type::<ToonMaterial>()
    // logic
    .add_systems(Startup, spawn_player)
    .add_systems(Startup, spawn_props)
    .add_systems(Startup, spawn_camera_and_lights)
    .add_systems(Update, utils::animate_light_direction)
    // .add_systems(Update, utils::animate_player_direction)
    .add_systems(Update, apply_user_movement)
    .run();
}
