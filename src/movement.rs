use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Indicates that an entity's movement is controlled by the user.
#[derive(Component)]
pub struct UserMovement;

/// Defines the max speed for an entity. All units are world units per second.
#[derive(Component)]
pub struct MaxSpeeds {
  speed: f32,
}

impl Default for MaxSpeeds {
  fn default() -> Self {
    Self { speed: 2.0 }
  }
}

/// Moves entities with `UserMovement` based on keyboard input.
pub fn apply_user_movement(
  time: Res<Time>,
  kb: Res<Input<KeyCode>>,
  mut query: Query<
    (
      &MaxSpeeds,
      &mut KinematicCharacterController,
      Option<&KinematicCharacterControllerOutput>,
    ),
    With<UserMovement>,
  >,
) {
  for (max_speeds, mut character, character_output) in query.iter_mut() {
    info!("character_output: {:?}", character_output);
    let mut direction = Vec3::default();
    if kb.pressed(KeyCode::W) {
      direction -= Vec3::Z;
    }
    if kb.pressed(KeyCode::S) {
      direction += Vec3::Z;
    }
    if kb.pressed(KeyCode::D) {
      direction += Vec3::X;
    }
    if kb.pressed(KeyCode::A) {
      direction -= Vec3::X;
    }
    if kb.pressed(KeyCode::Q) {
      direction -= Vec3::Y;
    }
    if kb.pressed(KeyCode::E) {
      direction += Vec3::Y;
    }

    // normalize if there are multiple inputs
    if direction.length_squared() > 0.0 {
      direction = direction.normalize() * max_speeds.speed;
    }

    // apply gravity
    if let Some(character_output) = character_output {
      if !character_output.grounded {
        direction += (Vec3::Y * -9.81)
          + (character_output.effective_translation / time.delta_seconds());
      }
    }

    character.translation = Some(time.delta_seconds() * direction);
  }
}
