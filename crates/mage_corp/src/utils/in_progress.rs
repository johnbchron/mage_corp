use bevy::{asset::Asset, prelude::*, tasks::Task};
use futures_lite::future::{block_on, poll_once};

/// A component for handling a task which returns a component.
///
/// When the task is completed, the return value is added as a component to the
/// entity containing the `InProgressComponent`, and the `InProgressComponent`
/// is removed.
///
/// Remember to schedule the system for your specific type.
#[derive(Component)]
pub struct InProgressComponent<T: Bundle>(pub Task<T>);

/// A component for handling a task which returns an asset.
///
/// When the task is completed, the return value is added to the corresponding
/// asset collection, the resulting handle is added as a component to the entity
/// containing the `InProgressAsset`, and the `InProgressAsset` is
/// removed.
///
/// Remember to schedule the system for your specific type.
#[derive(Component)]
pub struct InProgressAsset<T: Asset>(pub Task<T>);

/// Flushes the results from `InProgressComponent`.
pub fn in_progress_component_flusher<T: Bundle>(
  mut commands: Commands,
  mut query: Query<(Entity, &mut InProgressComponent<T>)>,
) {
  for (entity, mut in_progress) in &mut query {
    if let Some(component) = block_on(poll_once(&mut in_progress.0)) {
      commands.entity(entity).insert(component);
      commands.entity(entity).remove::<InProgressComponent<T>>();
    }
  }
}

/// Flushes the results from `InProgressAsset`.
pub fn in_progress_asset_flusher<T: Asset>(
  mut commands: Commands,
  mut query: Query<(Entity, &mut InProgressAsset<T>)>,
  mut asset_collection: ResMut<Assets<T>>,
) {
  for (entity, mut in_progress) in &mut query {
    if let Some(asset) = block_on(poll_once(&mut in_progress.0)) {
      commands.entity(entity).insert(asset_collection.add(asset));
      commands.entity(entity).remove::<InProgressAsset<T>>();
    }
  }
}
