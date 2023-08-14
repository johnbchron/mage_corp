use core::f32::consts::{FRAC_PI_4, PI};

use bevy::prelude::*;

#[allow(dead_code)]
pub fn animate_light_direction(
  time: Res<Time>,
  mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
  for mut transform in &mut query {
    transform.rotation = Quat::from_euler(
      EulerRot::ZYX,
      0.0,
      // time.elapsed_seconds() * PI / 10.0,
      16.0 * PI / 10.0,
      -FRAC_PI_4 / 2.0 * 3.0,
    );
  }
}
