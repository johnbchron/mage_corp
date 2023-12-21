use planiscope::shape::Shape;

pub enum Erosion {
  LikeNew,
  Worn,
  Damaged,
  Ruined,
}

pub enum Primitives {
  Plank {
    erosion:    Erosion,
    dimensions: glam::Vec3,
  },
  Brick {},
}

impl From<Primitives> for Shape {
  fn from(primitive: Primitives) -> Self {
    match primitive {
      Primitives::Plank {} => todo!(),
      Primitives::Brick {} => todo!(),
    }
  }
}
