use bevy::prelude::*;
use bevy_mod_wanderlust::ControllerInput;

/// Moves entities with `bevy_mod_wanderlust::ControllerInput` based on keyboard
/// input.
pub fn apply_movement_input(
  mut controller_query: Query<(&mut ControllerInput, &GlobalTransform)>,
  input: Res<Input<KeyCode>>,
) {
  for (mut controller_input, global_transform) in controller_query.iter_mut() {
    let mut dir = Vec3::ZERO;
    if input.pressed(KeyCode::A) {
      dir += -global_transform.right();
    }
    if input.pressed(KeyCode::D) {
      dir += global_transform.right();
    }
    if input.pressed(KeyCode::S) {
      dir += -global_transform.forward();
    }
    if input.pressed(KeyCode::W) {
      dir += global_transform.forward();
    }

    dir.y = 0.0;
    controller_input.movement = dir.normalize_or_zero();

    controller_input.jumping = input.pressed(KeyCode::Space);
  }
}
