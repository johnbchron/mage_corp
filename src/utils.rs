use core::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;

use crate::movement::UserMovement;

#[allow(dead_code)]
pub fn animate_light_direction(
  time: Res<Time>,
  mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
  for mut transform in &mut query {
    transform.rotation = Quat::from_euler(
      EulerRot::ZYX,
      0.0,
      time.elapsed_seconds() * PI / 10.0,
      -FRAC_PI_4 / 2.0 * 3.0,
    );
  }
}

#[allow(dead_code)]
pub fn animate_player_direction(
  time: Res<Time>,
  mut query: Query<&mut Transform, With<UserMovement>>,
) {
  for mut transform in &mut query {
    transform.rotation = Quat::from_euler(
      EulerRot::ZYX,
      0.0,
      time.elapsed_seconds() * PI / 5.0,
      0.0,
    );
  }
}
