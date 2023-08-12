mod low_res;
mod movement;
mod player;
mod scene;
mod toon;
mod utils;

use core::f32::consts::PI;

use bevy::{
  core_pipeline::{
    clear_color::ClearColorConfig,
    prepass::{DepthPrepass, NormalPrepass},
  },
  diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  prelude::*,
  render::camera::ScalingMode,
};
use bevy_diagnostic_vertex_count::{
  VertexCountDiagnosticsPlugin, VertexCountDiagnosticsSettings,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_wanderlust::WanderlustPlugin;
use bevy_rapier3d::prelude::*;

use crate::{
  low_res::{LowResCamera, LowResPlugin},
  movement::apply_movement_input,
  player::spawn_player,
  scene::ScenePlugin,
  toon::{ToonMaterial, ToonPlugin},
};

fn spawn_props(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn a plane below the player
  commands.spawn((
    MaterialMeshBundle {
      mesh: meshes.add(
        Mesh::try_from(shape::Box {
          min_x: -5.0,
          max_x: 5.0,
          min_y: -0.05,
          max_y: 0.05,
          min_z: -5.0,
          max_z: 5.0,
        })
        .unwrap(),
      ),
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(0.2, 0.2, 0.2),
        outline_scale: 0.0,
        ..default()
      }),
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
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(0.6, 0.2, 0.2),
        ..default()
      }),
      transform: Transform::from_xyz(-1.5, 1.0, 0.0),
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
      transform: Transform::from_xyz(
        8.0,
        (PI / 6.0).tan() * 8.0 * 2.0_f32.sqrt(),
        8.0,
      )
      // transform: Transform::from_xyz(
      //   0.0,
      //   8.0 * 2.0_f32.sqrt(),
      //   8.0,
      // )
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
    LowResCamera { pixel_size: 2 },
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
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
          }),
          ..Default::default()
        })
        .set(ImagePlugin::default_nearest()),
    )
    // graphics
    .add_plugins(ToonPlugin)
    .add_plugins(LowResPlugin)
    .insert_resource(Msaa::Off)
    // physics
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
    // .add_plugins(RapierDebugRenderPlugin::default())
    // movement
    .add_plugins(WanderlustPlugin)
    .add_systems(Update, apply_movement_input)
    // inspector
    .add_plugins(WorldInspectorPlugin::new())
    // QoL
    // setup
    // .add_plugins(ScenePlugin)
    .add_systems(Startup, spawn_player)
    .add_systems(Startup, spawn_props)
    .add_systems(Startup, spawn_camera_and_lights)
    // debug
    .add_systems(Update, utils::animate_light_direction)
    .add_plugins(LogDiagnosticsPlugin::default())
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .add_plugins(EntityCountDiagnosticsPlugin::default())
    .insert_resource(VertexCountDiagnosticsSettings { only_visible: true })
    .add_plugins(VertexCountDiagnosticsPlugin::default())
    .run();
}
