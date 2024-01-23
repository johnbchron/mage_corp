use std::f32::consts::PI;

use bevy::{
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  prelude::*,
};
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{
  camera::lowres::{LowresCamera, LowresCameraBundle},
  materials::ToonMaterial,
};

fn test_scene(
  mut commands: Commands,
  toon_materials: ResMut<Assets<ToonMaterial>>,
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

  // flat walls
  let mut comp = framix::Composition::new();
  comp.add_module(framix::BrickWall, IVec3::new(0, 2, 0));
  comp.add_module(framix::BrickWall, IVec3::new(1, 2, 1));
  comp.add_module(framix::BrickWall, IVec3::new(0, 2, 2));
  comp.add_module(framix::BrickWall, IVec3::new(-1, 2, 1));
  comp.spawn(&mut commands, toon_materials.into_inner());

  // let rendered_module = framix::BrickWall.render();
  // rendered_module.spawn(
  //   Transform::from_xyz(0.0, 2.0,
  // 0.0).with_rotation(Quat::from_rotation_y(PI)),   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(1.0, 2.0, 1.0)
  //     .with_rotation(Quat::from_rotation_y(PI / 2.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(0.0, 2.0, 2.0)
  //     .with_rotation(Quat::from_rotation_y(0.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(-1.0, 2.0, 1.0)
  //     .with_rotation(Quat::from_rotation_y(PI * 3.0 / 2.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // // corners
  // let rendered_module = framix::BrickCornerWall.render();
  // rendered_module.spawn(
  //   Transform::from_xyz(-1.0, 2.0, 0.0)
  //     .with_rotation(Quat::from_rotation_y(PI)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(1.0, 2.0, 0.0)
  //     .with_rotation(Quat::from_rotation_y(PI / 2.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(1.0, 2.0, 2.0)
  //     .with_rotation(Quat::from_rotation_y(0.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
  // rendered_module.spawn(
  //   Transform::from_xyz(-1.0, 2.0, 2.0)
  //     .with_rotation(Quat::from_rotation_y(PI * 3.0 / 2.0)),
  //   &mut commands,
  //   &mut toon_materials,
  // );
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) { app.add_systems(Startup, test_scene); }
}
