use bevy::prelude::*;

use crate::{
  camera::states::{
    maintain_state, CameraPureState, CameraState, CameraStateTarget,
  },
  materials::toon::ConvertToToonMaterial,
  terrain::TerrainDetailTarget,
};

fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
  commands
    .spawn((
      SpatialBundle::from_transform(Transform::from_xyz(-2.0, 0.0, 0.0)),
      CameraStateTarget,
      TerrainDetailTarget,
      UserInputReceiver { speed: 10.0 },
      Name::new("player"),
    ))
    .with_children(|parent| {
      parent.spawn((
        SceneBundle {
          scene: asset_server.load("scenes/fox.glb#Scene0"),
          transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_to(Vec3::Z, Vec3::Y),
          ..default()
        },
        ConvertToToonMaterial {
          outline_scale: Some(1.0),
          ..default()
        },
      ));
    });
}

#[derive(Component)]
pub struct UserInputReceiver {
  speed: f32,
}

pub fn simple_player_input(
  mut player_q: Query<(&mut Transform, &UserInputReceiver)>,
  kb_input: Res<Input<KeyCode>>,
  time: Res<Time>,
) {
  for (mut transform, user_input_receiver) in player_q.iter_mut() {
    let mut direction = Vec3::ZERO;
    if kb_input.pressed(KeyCode::W) {
      direction -= Vec3::Z;
    }
    if kb_input.pressed(KeyCode::A) {
      direction -= Vec3::X;
    }
    if kb_input.pressed(KeyCode::S) {
      direction += Vec3::Z;
    }
    if kb_input.pressed(KeyCode::D) {
      direction += Vec3::X;
    }

    direction = direction.normalize_or_zero()
      * time.delta_seconds()
      * user_input_receiver.speed;
    transform.translation += direction;
  }
}

fn debug_change_camera_states(
  kb_input: Res<Input<KeyCode>>,
  mut state_q: Query<&mut CameraState>,
) {
  if !kb_input.just_pressed(KeyCode::P) {
    return;
  }

  if let Ok(mut state) = state_q.get_single_mut() {
    match state.clone() {
      CameraState::InState(from) => match from {
        CameraPureState::Disabled => {}
        CameraPureState::OverShoulder => {}
        CameraPureState::Isometric => {
          state.transition(&CameraPureState::TestState)
        }
        CameraPureState::TestState => {
          state.transition(&CameraPureState::Isometric)
        }
      },
      CameraState::Transition { .. } => {
        state.reverse();
      }
    }
  }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, spawn_player)
      .add_systems(Update, simple_player_input.after(maintain_state))
      .add_systems(Update, debug_change_camera_states.after(maintain_state));
  }
}
