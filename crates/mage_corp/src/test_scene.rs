use std::f32::consts::PI;

use bevy::{
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  prelude::*,
};
use bevy_panorbit_camera::PanOrbitCamera;
use framix::Module;

use crate::{
  camera::lowres::{LowresCamera, LowresCameraBundle},
  materials::ToonMaterial,
};

fn test_scene(
  mut commands: Commands,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn the camera
  commands.spawn((
    LowresCameraBundle {
      lowres_camera: LowresCamera {
        min_pixel_scale: 2,
        final_far:       Some(10000.0),
        n_cameras:       1,
      },
      spatial_bundle: SpatialBundle::from_transform(
        Transform::from_xyz(0.0, 5.0, 10.0)
          .looking_at(Vec3::new(0.0, 6.0, 0.0), Vec3::Y),
      ),
      projection: Projection::Perspective(PerspectiveProjection {
        far: 250.0,
        ..default()
      }),
      ..default()
    },
    PanOrbitCamera::default(),
    NormalPrepass,
    DepthPrepass,
  ));

  // spawn a directional light
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadows_enabled: true,
      ..default()
    },
    transform: Transform {
      translation: Vec3::new(0.0, 2.0, 0.0),
      rotation: Quat::from_euler(EulerRot::XYZ, -PI / 4.0, -PI / 4.0, 0.0),
      ..default()
    },
    ..default()
  });

  // spawn the player
  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(0.0, 2.0, 0.0)),
    crate::terrain::TerrainDetailTarget,
    Name::new("player"),
    crate::markers::Player,
    magicore::source::Source::default(),
  ));

  let rendered_module = framix::BrickWall.render();
  rendered_module.spawn(
    Vec3::new(0.0, 2.0, 0.0),
    &mut commands,
    &mut toon_materials,
  );
  rendered_module.spawn(
    Vec3::new(1.0, 2.0, 0.0),
    &mut commands,
    &mut toon_materials,
  );
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) { app.add_systems(Startup, test_scene); }
}
