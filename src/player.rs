use bevy::{math::vec3, prelude::*};
use bevy_mod_wanderlust::{
  ControllerBundle, ControllerPhysicsBundle, ControllerSettings, Spring,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes};

use crate::ToonMaterial;

pub fn spawn_player(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  let material_handle = toon_materials.add(ToonMaterial::default());
  // let material_handle =
  // std_materials.add(StandardMaterial::from(Color::rgb(0.392, 0.584, 0.929)));
  let mesh_handle = meshes.add(Mesh::try_from(shape::Cube::new(1.0)).unwrap());

  commands.spawn((
    ControllerBundle {
      settings: ControllerSettings {
        acceleration: 25.0,
        max_speed: 10.0,
        max_acceleration_force: 10.0,
        up_vector: Vec3::Y,
        //gravity: -9.8,
        gravity: -20.0,
        max_ground_angle: 45.0 * (std::f32::consts::PI / 180.0),
        min_float_offset: -0.3,
        max_float_offset: 0.05,
        jump_time: 0.5,
        jump_initial_force: 8.0,
        jump_stop_force: 0.3,
        jump_decay_function: Some(|x| (1.0 - x).sqrt()),
        jump_skip_ground_check_duration: 0.0,
        coyote_time_duration: 0.16,
        jump_buffer_duration: 0.16,
        force_scale: vec3(1.0, 0.0, 1.0),
        float_cast_length: 1.0,
        float_cast_collider: Collider::ball(0.45),
        float_distance: 0.0,
        float_spring: Spring {
          strength: 500.0,
          damping:  1.0,
        },
        upright_spring: Spring {
          strength: 10.0,
          damping:  0.5,
        },
        ..default()
      },
      physics: ControllerPhysicsBundle {
        collider: Collider::cuboid(0.5, 0.5, 0.5),
        locked_axes: LockedAxes::ROTATION_LOCKED_X
          | LockedAxes::ROTATION_LOCKED_Y
          | LockedAxes::ROTATION_LOCKED_Z,
        ..default()
      },
      transform: Transform::from_xyz(1.5, 0.0, 0.0),
      ..default()
    },
    material_handle.clone(),
    mesh_handle.clone(),
  ));

  // commands.spawn(MaterialMeshBundle {
  //   material: material_handle.clone(),
  //   mesh: mesh_handle.clone(),
  //   ..default()
  // });
}
