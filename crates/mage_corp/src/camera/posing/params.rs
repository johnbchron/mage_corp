use bevy::prelude::*;

use super::super::low_res::LowResCamera;
use crate::utils::f32_lerp;

#[derive(Debug)]
pub struct ControlledCameraParams {
  pub translation:        Vec3,
  pub looking_at:         (Vec3, Vec3),
  pub fov:                f32,
  pub low_res_pixel_size: f32,
}

// we explicitly ignore the `looking_at` field in comparisons
impl PartialEq for ControlledCameraParams {
  fn eq(&self, other: &Self) -> bool {
    self.translation == other.translation
      && self.fov == other.fov
      && self.low_res_pixel_size == other.low_res_pixel_size
  }
}

impl ControlledCameraParams {
  pub fn from_components(
    transform: &Transform,
    projection: &Projection,
    lowres_camera: &LowResCamera,
  ) -> Option<Self> {
    match projection {
      Projection::Perspective(projection) => Some(Self {
        translation:        transform.translation,
        // we don't have the information to construct this, but it's mostly
        // irrelevant bc we won't use it in comparison
        looking_at:         (Vec3::ZERO, Vec3::ZERO),
        fov:                projection.fov,
        low_res_pixel_size: lowres_camera.pixel_size,
      }),
      Projection::Orthographic(_) => None,
    }
  }

  pub fn apply(
    &self,
    transform: &mut Transform,
    projection: &mut Projection,
    lowres_camera: &mut LowResCamera,
  ) {
    if let Projection::Perspective(ref mut projection) = projection {
      projection.fov = self.fov;
    }
    *transform = Transform::from_translation(self.translation)
      .looking_at(self.looking_at.0, self.looking_at.1);
    lowres_camera.pixel_size = self.low_res_pixel_size;
  }

  pub fn lerp(from: &Self, to: &Self, s: f32) -> Self {
    Self {
      translation:        from.translation.lerp(to.translation, s),
      looking_at:         to.looking_at,
      fov:                f32_lerp(from.fov, to.fov, s),
      low_res_pixel_size: f32_lerp(
        from.low_res_pixel_size,
        to.low_res_pixel_size,
        s,
      ),
    }
  }
}
