use std::f32::consts::PI;

use bevy::{
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  prelude::*,
};
use bevy_panorbit_camera::PanOrbitCamera;
use framix::{Direction, FragmentCoords};

use crate::camera::lowres::{LowresCamera, LowresCameraBundle};

fn test_scene(mut commands: Commands) {
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
}

fn spawn_framix_test(world: &mut World) {
  // flat walls
  let mut comp = framix::Composition::new();
  for y in 0..=1 {
    for a in 1..=3 {
      comp.add_fragment(
        framix::Fragment::BrickWall(framix::BrickWallFragment::Wall),
        FragmentCoords::new(IVec3::new(a, y, 0), Direction::South),
      );
      comp.add_fragment(
        framix::Fragment::BrickWall(framix::BrickWallFragment::Wall),
        FragmentCoords::new(IVec3::new(a, y, 4), Direction::North),
      );
      comp.add_fragment(
        framix::Fragment::BrickWall(framix::BrickWallFragment::Wall),
        FragmentCoords::new(IVec3::new(4, y, a), Direction::West),
      );
      comp.add_fragment(
        framix::Fragment::BrickWall(framix::BrickWallFragment::Wall),
        FragmentCoords::new(IVec3::new(0, y, a), Direction::East),
      );
    }
    comp.add_fragment(
      framix::Fragment::BrickWall(framix::BrickWallFragment::Corner),
      FragmentCoords::new(IVec3::new(0, y, 0), Direction::South),
    );
    comp.add_fragment(
      framix::Fragment::BrickWall(framix::BrickWallFragment::Corner),
      FragmentCoords::new(IVec3::new(4, y, 0), Direction::West),
    );
    comp.add_fragment(
      framix::Fragment::BrickWall(framix::BrickWallFragment::Corner),
      FragmentCoords::new(IVec3::new(4, y, 4), Direction::North),
    );
    comp.add_fragment(
      framix::Fragment::BrickWall(framix::BrickWallFragment::Corner),
      FragmentCoords::new(IVec3::new(0, y, 4), Direction::East),
    );
  }
  // concrete foundation
  for i in 0..=4 {
    for j in 0..=4 {
      comp.add_fragment(
        framix::Fragment::Foundation(framix::FoundationFragment),
        FragmentCoords::new(IVec3::new(i, -1, j), Direction::South),
      );
    }
  }
  comp.spawn(world, Transform::from_xyz(0.0, 1.0, 0.0));
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, (test_scene, spawn_framix_test));
  }
}
