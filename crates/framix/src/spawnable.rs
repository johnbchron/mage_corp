use bevy::prelude::*;

pub(crate) trait Spawnable {
  type SpawnContext: Clone + Send + Sync + 'static;

  fn spawn(&self, world: &mut World, context: Self::SpawnContext);
}
