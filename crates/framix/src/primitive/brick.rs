//! Brick primitives.

use super::*;

/// The dimensions of a standard(ish) brick in meters.
const STANDARD_BRICK_HALF_EXTENTS: glam::Vec3 =
  glam::Vec3::new(0.1, 0.025, 0.05);

/// A brick.
///
/// For now the brick is assumed to be a red facing brick.
#[derive(Reflect)]
pub struct Brick {
  /// The scale of the brick. A standard brick is 10cm x 2.5cm x 5cm.
  /// Defaults to `glam::Vec3::ONE`.
  pub scale: glam::Vec3,
}

impl Primitive for Brick {
  fn aabb(&self) -> Aabb {
    let half_dims = STANDARD_BRICK_HALF_EXTENTS * self.scale;
    Aabb::from_min_max(-half_dims, half_dims)
  }
  fn shape(&self) -> Shape {
    // convert to meters
    let dims = STANDARD_BRICK_HALF_EXTENTS;
    let outer_box = sb::cuboid(dims.x, dims.y, dims.z);

    let hole = sb::cylinder(0.2 * dims.x, dims.z * 2.2);
    let all_holes = sb::min(
      hole.clone(),
      sb::min(
        sb::translate(hole.clone(), (-dims.x * 0.55).into(), 0.0, 0.0),
        sb::translate(hole.clone(), (dims.x * 0.55).into(), 0.0, 0.0),
      ),
    );
    sb::scale(
      sb::max(outer_box, -all_holes),
      self.scale.x.into(),
      self.scale.y.into(),
      self.scale.z.into(),
    )
  }
  fn collider(&self) -> Option<Collider> {
    // adjust for the tessellation reducing the size just a bit
    let tess_cell_size = self.resolution().recip();
    let dims = STANDARD_BRICK_HALF_EXTENTS * 2.0 * self.scale - tess_cell_size;
    Some(Collider::cuboid(dims.x, dims.y, dims.z))
  }
  // source: https://www.engineeringtoolbox.com/bricks-density-d_1777.html
  fn density(&self) -> ColliderDensity { ColliderDensity(1765.0) }
  fn material(&self) -> ToonMaterial {
    ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::hex("#d49255").unwrap(),
        ..Default::default()
      },
      extension: ToonExtension::default(),
    }
  }
  fn friction(&self) -> Friction {
    Friction {
      static_coefficient: 0.7,
      dynamic_coefficient: 0.6,
      ..Default::default()
    }
  }
  fn restitution(&self) -> Restitution { Restitution::new(0.05) }
}

impl Default for Brick {
  fn default() -> Self {
    Brick {
      scale: glam::Vec3::ONE,
    }
  }
}
