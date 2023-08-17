mod movement;

use bevy::{math::vec3, prelude::*};
use bevy_mod_wanderlust::{
  ControllerBundle, ControllerPhysicsBundle, ControllerSettings, Spring,
  WanderlustPlugin,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes};

use crate::materials::toon::ConvertToToonMaterial;

pub fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
  commands
    .spawn((
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
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
      },
      Name::new("player"),
    ))
    .with_children(|parent| {
      parent.spawn((
        SceneBundle {
          scene: asset_server.load("scenes/fox.glb#Scene0"),
          transform: Transform::from_xyz(0.0, -0.5, 0.0),
          ..default()
        },
        ConvertToToonMaterial {
          outline_scale: Some(1.0),
          ..default()
        },
      ));
    });
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(WanderlustPlugin)
      .add_systems(Startup, spawn_player)
      .add_systems(Update, movement::apply_movement_input);
  }
}
