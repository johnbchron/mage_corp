use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
  camera::lowres::LowresCamera,
  materials::{ToonExtension, ToonMaterial},
};

fn test_scene(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  // spawn the camera
  commands.spawn((
    LowresCamera {
      n_cameras: 1,
      min_pixel_scale: 4,
      ..default()
    },
    SpatialBundle::from_transform(
      Transform::from_xyz(0.0, 5.0, 10.0)
        .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ),
    Name::new("lowres_camera"),
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
    toon_materials.add(
      ToonMaterial {
        base:      StandardMaterial {
          base_color: Color::rgb(0.8, 0.7, 0.6),
          perceptual_roughness: 0.2,
          reflectance: 0.1,
          ..default()
        },
        extension: ToonExtension::default(),
      }
      .into(),
    ),
    crate::terrain::TerrainDetailTarget,
    Name::new("sphere"),
  ));
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, test_scene);
  }
}
