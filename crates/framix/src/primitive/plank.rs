//! Wood plank primitives.

use super::*;

/// A plank of wood.
///
/// For now the wood species is assumed to be White American Oak.
#[derive(Reflect)]
pub struct Plank {
  /// The dimensions of the plank in meters.
  pub dims:      glam::Vec3,
  /// The grain direction of the plank.
  pub grain_dir: glam::Vec3,
}

impl Primitive for Plank {
  fn aabb(&self) -> Aabb {
    let half_dims = self.dims * 0.5;
    Aabb::from_min_max(-half_dims, half_dims)
  }
  fn shape(&self) -> Shape { sb::cuboid(self.dims.x, self.dims.y, self.dims.z) }
  fn collider(&self) -> Option<Collider> {
    Some(Collider::cuboid(self.dims.x, self.dims.y, self.dims.y))
  }
  // https://www.engineeringtoolbox.com/wood-density-d_40.html
  fn density(&self) -> ColliderDensity { ColliderDensity(790.0) }
  fn material(&self) -> ToonMaterial {
    ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::hex("#b5651d").unwrap(),
        ..Default::default()
      },
      extension: ToonExtension::default(),
    }
  }
  fn friction(&self) -> Friction { todo!() }
  fn restitution(&self) -> Restitution { todo!() }
}
