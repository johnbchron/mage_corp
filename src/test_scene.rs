use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{
  core_pipeline::{
    clear_color::ClearColorConfig,
    prepass::{DepthPrepass, NormalPrepass},
  },
  prelude::*,
  render::camera::ScalingMode,
};
use bevy_rapier3d::prelude::Collider;

use crate::{
  low_res::LowResCamera,
  particle::{
    descriptor::{
      ParticleBehavior, ParticleDescriptor, ParticleLinearVelocity,
      ParticleSizeBehavior,
    },
    ParticleEmitter, ParticleEmitterRegion,
  },
  toon::ToonMaterial,
};

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_camera_and_lights)
      .add_systems(Startup, setup_scene_props)
      .add_systems(Startup, setup_particle_emitter)
      .add_systems(Startup, setup_translucent_ball);
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
        8.0,
        (PI / 6.0).tan() * 8.0 * 2.0_f32.sqrt(),
        8.0,
      )
      .looking_at(Vec3::default(), Vec3::Y),
      projection: OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize(50.0),
        ..default()
      }
      .into(),
      ..default()
    },
    DepthPrepass,
    NormalPrepass,
    LowResCamera { pixel_size: 4 },
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

fn setup_scene_props(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn a plane below the player
  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(
        Mesh::try_from(shape::Box {
          min_x: -500.0,
          max_x: 500.0,
          min_y: -0.05,
          max_y: 0.05,
          min_z: -500.0,
          max_z: 500.0,
        })
        .unwrap(),
      ),
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(0.180, 0.267, 0.169),
        outline_scale: 0.0,
        ..default()
      }),
      transform: Transform::from_xyz(0.0, -0.5, 0.0),
      ..default()
    },
    Collider::cuboid(500.0, 0.05, 500.0),
    Name::new("ground_plane"),
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
  mut std_materials: ResMut<Assets<StandardMaterial>>,
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
      material: std_materials.add(StandardMaterial::from(Color::rgba(
        0.392, 0.584, 0.929, 0.2,
      ))),
      transform: Transform::from_xyz(1.5, 0.0, 0.0),
      ..default()
    },
    Collider::cuboid(0.5, 0.5, 0.5),
    Name::new("translucent_ball"),
  ));
}