//! Concrete block primitive.

use super::*;

/// A concrete block.
#[derive(Reflect)]
pub struct ConcreteBlock {
  /// The dimensions of the block.
  pub dims: Vec3,
}

impl Primitive for ConcreteBlock {
  fn aabb(&self) -> Aabb {
    let half_dims = Vec3::splat(0.5) * self.dims;
    Aabb::from_min_max(-half_dims, half_dims)
  }
  fn shape(&self) -> Shape {
    let dims = Vec3::splat(0.5) * self.dims;
    sb::cuboid(dims.x, dims.y, dims.z)
  }
  fn collider(&self) -> Option<Collider> {
    let dims = Vec3::splat(0.5) * self.dims;
    Some(Collider::cuboid(dims.x, dims.y, dims.z))
  }
  fn material(&self) -> ToonMaterial {
    ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::hex("#C0C0C0").unwrap(),
        ..Default::default()
      },
      extension: ToonExtension::default(),
    }
  }
  fn density(&self) -> ColliderDensity { ColliderDensity(2400.0) }
  fn friction(&self) -> Friction {
    Friction {
      static_coefficient: 0.7,
      dynamic_coefficient: 0.6,
      ..Default::default()
    }
  }
  fn restitution(&self) -> Restitution { Restitution::new(0.05) }
}

impl Default for ConcreteBlock {
  fn default() -> Self {
    Self {
      dims: Vec3::new(1.0, 1.0, 1.0),
    }
  }
}
