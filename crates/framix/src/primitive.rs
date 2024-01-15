use bevy::{
  pbr::StandardMaterial,
  render::{color::Color, primitives::Aabb},
};
use bevy_implicits::prelude::{builder as sb, Shape};
use bevy_xpbd_3d::components::{Collider, Friction, Restitution};
use common::materials::{ToonExtension, ToonMaterial};

pub trait Primitive {
  /// The [`AABB`] of the primitive.
  fn aabb(&self) -> Aabb;
  /// The [`Shape`] of the primitive.
  fn shape(&self) -> Shape;
  /// The [`Shape`] to use for calculating the primitive's [Collider], in case
  /// `collider_shape` returns `None`.
  fn collider_shape(&self) -> Shape { self.shape() }
  /// The [`Collider`] of the primitive, if it's feasible to generate directly.
  /// Otherwise return `None` and the `collider_shape` will be used in a convex
  /// decomposition to generate the collider.
  fn collider(&self) -> Option<Collider> { None }
  /// The resolution at which to tessellate the primitive.
  fn resolution(&self) -> f32 { 500.0 }
  /// The [`ToonMaterial`] of the primitive.
  fn material(&self) -> ToonMaterial;
  /// The density of the primitive.
  fn density(&self) -> f32;
  /// The friction properties of the primitive.
  fn friction(&self) -> Friction;
  /// The restitution properties of the primitive.
  fn restitution(&self) -> Restitution;
}

/// A plank of wood.
///
/// For now the wood species is assumed to be White American Oak.
pub struct Plank {
  pub dims:      glam::Vec3,
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
  fn density(&self) -> f32 { 790.0 }
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

/// The dimensions of a standard(ish) brick in meters.
const STANDARD_BRICK_HALF_EXTENTS: glam::Vec3 =
  glam::Vec3::new(0.1, 0.025, 0.05);

/// A brick.
///
/// For now the brick is assumed to be a red facing brick.
pub struct Brick {
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
  fn density(&self) -> f32 { 1765.0 }
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
