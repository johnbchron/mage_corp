use bevy::render::primitives::Aabb;
use bevy_implicits::prelude::{builder as sb, Shape};

pub trait Primitive {
  fn aabb(&self) -> Aabb;
  fn into_shape(&self) -> Shape;
  fn collider(&self) -> Shape { self.into_shape() }
}

pub struct Plank {
  pub dims:      glam::Vec3,
  pub grain_dir: glam::Vec3,
}

impl Primitive for Plank {
  fn aabb(&self) -> Aabb {
    let half_dims = self.dims * 0.5;
    Aabb::from_min_max((-half_dims).into(), half_dims.into())
  }

  fn into_shape(&self) -> Shape {
    sb::cuboid(self.dims.x, self.dims.y, self.dims.z)
  }
}

/// The dimensions of a standard brick in inches.
const STANDARD_BRICK_DIMENSIONS: glam::Vec3 =
  glam::Vec3::new(7.625, 2.25, 3.625);

pub struct Brick {
  pub scale: glam::Vec3,
}

impl Primitive for Brick {
  fn aabb(&self) -> Aabb {
    let dims = STANDARD_BRICK_DIMENSIONS * 0.0254;
    let half_dims = dims * self.scale * 0.5;
    Aabb::from_min_max((-half_dims).into(), half_dims.into())
  }

  fn into_shape(&self) -> Shape {
    // convert to meters
    let dims = STANDARD_BRICK_DIMENSIONS * 0.0254;
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

  fn collider(&self) -> Shape {
    let dims = STANDARD_BRICK_DIMENSIONS * 0.0254;
    let outer_box = sb::cuboid(dims.x, dims.y, dims.z);
    sb::scale(
      outer_box,
      self.scale.x.into(),
      self.scale.y.into(),
      self.scale.z.into(),
    )
  }
}
