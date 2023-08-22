
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, ActiveCameraData};
use super::*;

pub struct LowResPanOrbitCompatPlugin;

impl Plugin for LowResPanOrbitCompatPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Update, maintain_active_data.run_if(window_size_changed));
	}
}

fn maintain_active_data(
	camera_q: Query<(Entity, &LowResCamera), With<PanOrbitCamera>>,
	window_q: Query<&Window, With<PrimaryWindow>>,
	mut active_camera_data: ResMut<ActiveCameraData>,
) {
	if let Some((entity, lowres_camera)) = camera_q.iter().next() {
		let window = window_q.single();
		
		let texture_size = calculate_texture_resolution(window.width(), window.height(), lowres_camera.pixel_size);
		
		active_camera_data.set_if_neq(ActiveCameraData {
			entity: Some(entity),
			viewport_size: Some(texture_size),
			window_size: Some(Vec2::new(window.width(), window.height())),
			manual: true,
		});
	}
}