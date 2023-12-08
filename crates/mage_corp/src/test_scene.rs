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
  mut std_materials: ResMut<Assets<StandardMaterial>>,
) {
  // spawn the camera
  commands.spawn((
    LowresCamera::from_n_cameras(1, PerspectiveProjection {
      near: 1.0,
      far: 1000.0,
      ..default()
    }),
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
      rotation: Quat::from_rotation_x(-PI / 4.),
      ..default()
    },
    ..default()
  });

  // spawn a cube
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
        extension: ToonExtension { quantize_steps: 1 },
      }
      .into(),
    ),
    Name::new("cube"),
  ));

  // spawn a green ground plane
  commands.spawn((
    PbrBundle {
      mesh: meshes.add(Mesh::from(shape::Plane {
        size: 100.0,
        ..default()
      })),
      material: std_materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..Default::default()
    },
    Name::new("ground"),
  ));
}

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, test_scene);
  }
}
