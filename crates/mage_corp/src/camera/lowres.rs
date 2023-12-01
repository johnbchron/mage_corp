use std::ops::RangeInclusive;

use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct LowresCameraConfig {
  /// A list of depth ranges and their corresponding resolution. The ranges
  /// should be in increasing order, and should cover the entire range [0.0,
  /// 1.0].
  pub configs:      Vec<(RangeInclusive<f32>, u8)>,
  pub overall_proj: PerspectiveProjection,
}

impl LowresCameraConfig {
  /// Constructs a LowresCamera from a number of cameras, near and far.
  ///
  /// Each successive camera will have double the linear depth of the previous,
  /// and 1 pixel less resolution. The last camera will have 2-pixel resolution.
  fn from_n_cameras(n: u8, proj: PerspectiveProjection) -> Self {
    let total_max = 2_u32.pow(n as u32) - 1;
    let configs = (0..n)
      .map(|i| {
        let min = 2_u32.pow(i as u32) - 1;
        let max = 2_u32.pow((i + 1) as u32) - 1;
        let min = min as f32 / total_max as f32;
        let max = max as f32 / total_max as f32;
        (min..=max, n - i + 1)
      })
      .collect();
    Self {
      configs,
      overall_proj: proj,
    }
  }
}

impl Default for LowresCameraConfig {
  fn default() -> Self {
    Self::from_n_cameras(4, PerspectiveProjection::default())
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
  fn from_n_cameras_works() {
    let lowres_camera =
      LowresCameraConfig::from_n_cameras(3, PerspectiveProjection::default());
    let expected_configs = vec![
      (0.0..=(1.0 / 7.0), 4),
      ((1.0 / 7.0)..=(3.0 / 7.0), 3),
      ((3.0 / 7.0)..=(7.0 / 7.0), 2),
    ];
    assert_eq!(lowres_camera.configs, expected_configs);

    let lowres_camera =
      LowresCameraConfig::from_n_cameras(4, PerspectiveProjection::default());
    let expected_configs = vec![
      (0.0..=(1.0 / 15.0), 5),
      ((1.0 / 15.0)..=(3.0 / 15.0), 4),
      ((3.0 / 15.0)..=(7.0 / 15.0), 3),
      ((7.0 / 15.0)..=(15.0 / 15.0), 2),
    ];
    assert_eq!(lowres_camera.configs, expected_configs);
  }

  #[test]
  fn child_cameras_are_spawned() {
    let mut app = App::new();
    app.add_plugins(LowresCameraPlugin);
    let lowres_camera = app
      .world
      .spawn((SpatialBundle::default(), LowresCameraConfig::default()))
      .id();

    dbg!(LowresCameraConfig::default());

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
