use planiscope::shape::{builder as sb, Shape};

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
    grain_dir:  glam::Vec3,
  },
  Brick {
    erosion: Erosion,
    scale:   glam::Vec3,
  },
}

impl Primitives {
  fn outer_dimensions(&self) -> glam::Vec3 {
    match self {
      Primitives::Plank { dimensions, .. } => *dimensions,
      Primitives::Brick { scale, .. } => {
        let imperial_dimensions = glam::Vec3::new(7.625, 2.25, 3.625);
        let dimensions = imperial_dimensions * 0.0254;
        dimensions * *scale
      }
    }
  }
}

impl From<Primitives> for Shape {
  fn from(primitive: Primitives) -> Self {
    match primitive {
      Primitives::Plank { dimensions, .. } => {
        sb::cuboid(dimensions.x, dimensions.y, dimensions.z)
      }
      Primitives::Brick { scale, .. } => {
        let imperial_dimensions = glam::Vec3::new(7.625, 2.25, 3.625);
        let dimensions = imperial_dimensions * 0.0254;
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
        let brick = sb::scale(
          sb::max(outer_box, -all_holes),
          scale.x.into(),
          scale.y.into(),
          scale.z.into(),
        );
        brick
      }
    }
  }
}

pub fn brick_array(x: usize, y: usize) -> Shape {
  let brick = Primitives::Brick {
    erosion: Erosion::LikeNew,
    scale:   glam::Vec3::splat(1.0),
  };
  let brick_outer_dimensions = brick.outer_dimensions();
  let brick: Shape = brick.into();
  let smudge = 0.01;

  let mut accumulator = vec![];
  for i in 0..x {
    for j in 0..y {
      let mut transform = glam::Vec3::new(
        brick_outer_dimensions.x * 2.0 * i as f32 + smudge * i as f32,
        brick_outer_dimensions.y * 2.0 * j as f32 + smudge * j as f32,
        0.0,
      );
      if j % 2 == 1 {
        transform.x += brick_outer_dimensions.x;
      }
      accumulator.push(sb::translate(
        brick.clone(),
        transform.x.into(),
        transform.y.into(),
        transform.z.into(),
      ));
    }
  }

  let bricks = accumulator
    .into_iter()
    .reduce(|a, b| sb::min(a, b))
    .unwrap();
  bricks
}
