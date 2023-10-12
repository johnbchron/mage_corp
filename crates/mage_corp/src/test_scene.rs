#![allow(dead_code)]

use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{
  core_pipeline::{
    clear_color::ClearColorConfig,
    prepass::{DepthPrepass, NormalPrepass},
  },
  pbr::NotShadowCaster,
  prelude::*,
};
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{
  camera::{
    low_res::LowResCamera,
    posing::{CameraPose, CameraPoseState},
  },
  markers::MainCamera,
  materials::{
    force::ForceMaterial,
    toon::{ConvertToToonMaterial, ToonMaterial},
  },
  particle::{
    descriptor::{
      ParticleBehavior, ParticleDescriptor, ParticleLinearVelocity,
      ParticleSizeBehavior,
    },
    ParticleEmitter, ParticleEmitterRegion,
  },
};

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera_and_lights)
      // .add_plugins(bevy_obj::ObjPlugin)
      // .add_systems(Startup, setup_barn_scene)
      .add_systems(Startup, setup_particle_emitter)
      // .add_systems(Startup, setup_translucent_ball)
      // .add_systems(Startup, setup_npc_scene)
      .add_systems(Startup, setup_tree_scene);
  }
}

fn setup_camera_and_lights(mut commands: Commands) {
  commands.spawn((
    Camera3dBundle {
      camera_3d: Camera3d {
        clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
        ..default()
      },
      transform: Transform::from_xyz(
        64.0,
        (PI / 6.0).tan() * 64.0 * 2.0_f32.sqrt(),
        64.0,
      )
      .looking_at(Vec3::default(), Vec3::Y),
      projection: PerspectiveProjection {
        fov: 0.2,
        ..default()
      }
      .into(),
      ..default()
    },
    DepthPrepass,
    NormalPrepass,
    MainCamera,
    LowResCamera { pixel_size: 4.0 },
    CameraPoseState::InState(CameraPose::Disabled),
    PanOrbitCamera { ..default() },
    Name::new("lowres_camera"),
  ));

  commands.spawn((
    DirectionalLightBundle {
      directional_light: DirectionalLight {
        shadows_enabled: true,
        illuminance: 10000.0,
        ..default()
      },
      transform: Transform::from_rotation(Quat::from_euler(
        EulerRot::ZYX,
        0.0,
        16.0 * PI / 10.0,
        -FRAC_PI_4 / 2.0 * 3.0,
      )),
      ..default()
    },
    Name::new("directional_light"),
  ));
}

fn setup_particle_emitter(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  commands.spawn((
    SpatialBundle {
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..default()
    },
    ParticleEmitter::new(
      ParticleDescriptor {
        size:     0.1,
        material: toon_materials.add(ToonMaterial {
          color: Color::rgb(1.0, 0.5, 0.0),
          outline_scale: 0.0,
          ..default()
        }),
        shape:    meshes.add(
          Mesh::try_from(shape::Icosphere {
            radius:       0.5,
            subdivisions: 0,
          })
          .unwrap(),
        ),
        behavior: ParticleBehavior {
          initial_linear_velocity: ParticleLinearVelocity::Conic {
            cone_angle: 30.0,
            direction:  Vec3::Y,
            magnitude:  5.0,
          },
          lifetime: std::time::Duration::from_secs_f32(2.0),
          size_behavior: ParticleSizeBehavior::LinearShrink,
          ..default()
        },
      },
      ParticleEmitterRegion::Point { offset: None },
      100.0,
      false,
    ),
    Name::new("particle_emitter"),
  ));
}

fn setup_translucent_ball(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  // mut toon_materials: ResMut<Assets<ToonMaterial>>,
  // mut std_materials: ResMut<Assets<StandardMaterial>>,
  mut force_materials: ResMut<Assets<ForceMaterial>>,
) {
  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(
        Mesh::try_from(shape::Icosphere {
          radius:       0.5,
          subdivisions: 3,
        })
        .unwrap(),
      ),
      material: force_materials
        .add(ForceMaterial::from(Color::rgb(0.392, 0.584, 0.929))),
      transform: Transform::from_xyz(1.5, 0.0, 0.0),
      ..default()
    },
    NotShadowCaster,
    Name::new("translucent_ball"),
  ));
}

fn setup_npc_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((
    SceneBundle {
      scene: asset_server.load("scenes/boy.glb#Scene0"),
      transform: Transform::from_xyz(0.0, -0.5, 0.0),
      ..default()
    },
    ConvertToToonMaterial {
      outline_scale: Some(1.0),
      ..default()
    },
    Name::new("npc_boy"),
  ));
}

fn setup_barn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((
    SceneBundle {
      scene: asset_server.load("models/Barn.obj"),
      transform: Transform::from_xyz(0.0, -0.5, 0.0),
      ..default()
    },
    ConvertToToonMaterial {
      outline_scale: Some(1.0),
      ..default()
    },
    Name::new("barn"),
  ));
}

fn setup_tree_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((
    SceneBundle {
      scene: asset_server.load("scenes/tree.glb#Scene0"),
      transform: Transform::from_xyz(0.0, -0.5, 0.0),
      ..default()
    },
    ConvertToToonMaterial {
      outline_scale: Some(1.0),
      ..default()
    },
    Name::new("tree"),
  ));
}
