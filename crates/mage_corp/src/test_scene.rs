use std::f32::consts::PI;

use bevy::{
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  pbr::wireframe::Wireframe,
  prelude::*,
};
use bevy_implicits::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_xpbd_3d::prelude::*;

use crate::{
  camera::lowres::{LowresCamera, LowresCameraBundle},
  materials::{ToonExtension, ToonMaterial},
};

fn test_scene(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn the camera
  commands.spawn((
    LowresCameraBundle {
      lowres_camera: LowresCamera {
        min_pixel_scale: 2,
        final_far:       Some(10000.0),
        n_cameras:       4,
      },
      spatial_bundle: SpatialBundle::from_transform(
        Transform::from_xyz(0.0, 5.0, 10.0)
          .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
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

  // spawn a sphere
  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(0.0, 2.0, 0.0)),
    meshes.add(
      Mesh::try_from(shape::Icosphere {
        radius:       0.5,
        subdivisions: 4,
      })
      .unwrap(),
    ),
    toon_materials.add(ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        perceptual_roughness: 0.2,
        reflectance: 0.1,
        ..default()
      },
      extension: ToonExtension::default(),
    }),
    crate::terrain::TerrainDetailTarget,
    // RigidBody::Dynamic,
    Collider::ball(0.5),
    Name::new("sphere"),
    crate::markers::Player,
    magicore::source::Source::default(),
  ));

  // spawn a test for the implicits plugin
  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(0.0, 1.0, 0.0)),
    toon_materials.add(ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::hex("#b5651d").unwrap(),
        perceptual_roughness: 0.2,
        reflectance: 0.1,
        ..default()
      },
      extension: ToonExtension::default(),
    }),
    ImplicitInputs(MesherInputs {
      shape:        framix::brick_array(1, 1),
      region:       MesherRegion {
        position: Vec3::splat(0.0).into(),
        scale:    Vec3::splat(0.25).into(),
        detail:   MesherDetail::Resolution(200.0),
        prune:    false,
      },
      gen_collider: true,
    }),
    SyncImplicits,
    Wireframe,
    Name::new("implicits_test"),
  ));
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) { app.add_systems(Startup, test_scene); }
}
