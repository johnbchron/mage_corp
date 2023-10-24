pub mod blueprint;
pub mod source;

use bevy::prelude::*;
use blueprint::Blueprint;

pub struct MagicPlugin;

impl Plugin for MagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(blueprint::BlueprintPlugin)
      .add_systems(Startup, magic_test_scene);
  }
}

fn magic_test_scene(mut commands: Commands) {
  commands.spawn((
    SpatialBundle {
      transform: Transform::from_xyz(0.0, 3.0, 3.0),
      ..default()
    },
    Blueprint::new(blueprint::BlueprintType::MassBarrier),
    Name::new("blueprint_test"),
  ));
}
