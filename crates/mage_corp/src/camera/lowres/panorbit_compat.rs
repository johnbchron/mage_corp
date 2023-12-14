use bevy::prelude::*;
use bevy_panorbit_camera::{ActiveCameraData, PanOrbitCamera};

use super::*;

pub struct LowResPanOrbitCompatPlugin;

impl Plugin for LowResPanOrbitCompatPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      maintain_active_data.run_if(on_event::<RebuildEvent>()),
    );
  }
}

fn maintain_active_data(
  camera_q: Query<Entity, (With<PanOrbitCamera>, With<LowresCamera>)>,
  window_q: Query<&Window, With<PrimaryWindow>>,
  active_camera_data: Option<ResMut<ActiveCameraData>>,
) {
  if active_camera_data.is_none() {
    return;
  }
  let mut active_camera_data = active_camera_data.unwrap();

  if let Some(entity) = camera_q.iter().next() {
    let window = window_q.single();

    let window_size = Vec2::new(window.width(), window.height());

    active_camera_data.set_if_neq(ActiveCameraData {
      entity:        Some(entity),
      viewport_size: Some(window_size),
      window_size:   Some(window_size),
      manual:        true,
    });
  }
}
