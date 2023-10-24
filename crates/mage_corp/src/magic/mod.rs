pub mod blueprint;
pub mod source;

use bevy::prelude::*;
use blueprint::Blueprint;

pub struct MagicPlugin;

impl Plugin for MagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<blueprint::visuals::BlueprintVisualsPrefabs>()
      .register_type::<source::Source>()
      .register_type::<Blueprint>()
      .add_systems(Startup, magic_test_scene)
      .add_systems(Update, blueprint::visuals::maintain_blueprint_visuals);
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
