use std::ops::Range;

use bevy::prelude::*;

#[derive(Component)]
pub struct LowresCamera {
  pub configs: Vec<(Range<f32>, u8)>,
  pub near:    f32,
  pub far:     f32,
}

impl LowresCamera {
  fn from_n_cameras(n: u8, near: f32, far: f32) -> Self {
    let max_unit = 2_u32.pow(n as u32) - 1;
    let configs = (0..n)
      .map(|i| {
        let min = 2_f32.powf(i as f32) / max_unit as f32;
        let max = 2_f32.powf((i + 1) as f32) / max_unit as f32;
        let pixel_size = n - (i + 1) + 2;
        (min..max, pixel_size)
      })
      .collect();
    Self { configs, near, far }
  }
}

impl Default for LowresCamera {
  fn default() -> Self {
    Self::from_n_cameras(4, 0.1, 1000.0)
  }
}

pub struct LowresCameraPlugin;

impl Plugin for LowresCameraPlugin {
  fn build(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
  use bevy::prelude::*;

  use super::*;

  #[test]
  fn child_cameras_are_spawned() {
    let mut app = App::new();
    app.add_plugins(LowresCameraPlugin);
    let lowres_camera = app
      .world
      .spawn((SpatialBundle::default(), LowresCamera::default()))
      .id();
    app.update();
    let lowres_camera = app.world.get_entity(lowres_camera).unwrap();
    assert!(
      lowres_camera.get::<Children>().is_some(),
      "LowresCamera should have children"
    );
    let children = lowres_camera.get::<Children>().unwrap();
    for child in children.iter() {
      let child = app.world.get_entity(*child).unwrap();
      assert!(
        child.get::<Camera>().is_some(),
        "LowresCamera children should have cameras"
      );
    }
  }
}
