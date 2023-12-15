use bevy::prelude::*;

use super::{source::Source, spell::SourceLink};

#[derive(Component, Clone, Reflect)]
pub enum Target {
  RelativeCoords(Vec3),
}

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Target>()
      .add_systems(Update, eval_targets);
  }
}

fn eval_targets(
  mut commands: Commands,
  target_q: Query<(Entity, &Target, &SourceLink), Without<Transform>>,
  source_q: Query<&Transform, With<Source>>,
) {
  for (entity, target, source_link) in target_q.iter() {
    let source_transform = source_q.get(source_link.0).unwrap();

    match target {
      Target::RelativeCoords(coords) => {
        commands
          .entity(entity)
          .insert(SpatialBundle::from_transform(Transform::from_translation(
            *coords + source_transform.translation,
          )));
      }
    }
  }
}
