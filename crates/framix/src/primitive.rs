use bevy::render::primitives::Aabb;
use bevy_implicits::prelude::{builder as sb, Shape};

/// The dimensions of a standard brick in inches.
const STANDARD_BRICK_DIMENSIONS: glam::Vec3 =
  glam::Vec3::new(7.625, 2.25, 3.625);

pub enum Primitive {
  Plank {
    dimensions: glam::Vec3,
    grain_dir:  glam::Vec3,
  },
  Brick {
    scale: glam::Vec3,
  },
}

impl Primitive {
  fn aabb(&self) -> Aabb {
    match self {
      Primitive::Plank { dimensions, .. } => {
        let half_dimensions = *dimensions * 0.5;
        Aabb::from_min_max((-half_dimensions).into(), half_dimensions.into())
      }
      Primitive::Brick { scale } => {
        let half_dimensions = glam::Vec3::new(0.5, 0.5, 0.5) * *scale;
        Aabb::from_min_max((-half_dimensions).into(), half_dimensions.into())
      }
    }
  }

  fn into_shape(self: Primitive) -> Shape {
    match self {
      Primitive::Plank { dimensions, .. } => {
        sb::cuboid(dimensions.x, dimensions.y, dimensions.z)
      }
      Primitive::Brick { scale, .. } => {
        // convert to meters
        let dimensions = STANDARD_BRICK_DIMENSIONS * 0.0254;
        let outer_box = sb::cuboid(dimensions.x, dimensions.y, dimensions.z);

        let hole = sb::cylinder(0.2 * dimensions.x, dimensions.z * 2.2);
        let all_holes = sb::min(
          hole.clone(),
          sb::min(
            sb::translate(
              hole.clone(),
              (-dimensions.x * 0.55).into(),
              0.0,
              0.0,
            ),
            sb::translate(hole.clone(), (dimensions.x * 0.55).into(), 0.0, 0.0),
          ),
        );
        sb::scale(
          sb::max(outer_box, -all_holes),
          scale.x.into(),
          scale.y.into(),
          scale.z.into(),
        )
      }
    }
  }
}
