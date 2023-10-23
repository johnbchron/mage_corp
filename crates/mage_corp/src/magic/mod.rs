mod visuals;

use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Blueprint {
  _type:         BlueprintType,
  stage:         BlueprintStage,
  linked_source: Option<Entity>,
}

#[derive(Debug)]
enum BlueprintType {
  MassBarrier,
}

#[derive(Debug)]
enum BlueprintStage {
  Initialized,
  Built,
}

pub struct MagicPlugin;

impl Plugin for MagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<visuals::BlueprintVisualsPrefabs>()
      .add_systems(Startup, magic_test_scene)
      .add_systems(Update, visuals::maintain_blueprint_visuals);
  }
}

fn magic_test_scene(mut commands: Commands) {
  commands.spawn((
    SpatialBundle {
      transform: Transform::from_xyz(0.0, 3.0, 3.0),
      ..default()
    },
    Blueprint {
      _type:         BlueprintType::MassBarrier,
      stage:         BlueprintStage::Initialized,
      linked_source: None,
    },
  ));
}
