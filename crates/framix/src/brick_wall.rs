//! Brick wall fragments.

use std::f32::consts::PI;

use super::*;
use crate::rendered::RenderedPrimitive;

/// A brick wall fragment.
#[derive(Reflect, Default)]
pub enum BrickWallFragment {
  #[default]
  /// A brick wall fragment.
  Wall,
  /// A brick wall corner fragment.
  Corner,
}

impl FragmentConfig for BrickWallFragment {
  fn render(&self) -> RenderedFragment {
    // we'll fit 5 bricks end to end and 20 stacked. we'll also offset every
    // other brick by one half length.
    let smudge = 1.02;
    let mut rows = Vec::new();

    // honestly, this is barely comprehensible. it's almost not worth debugging.
    // if you encounter an issue, consider rewriting.
    match self {
      Self::Wall => {
        for i in 0..20 {
          let mut row = Vec::new();
          for j in 0..5 {
            row.push(RenderedPrimitive::new(
              Box::new(Brick {
                scale: glam::Vec3::splat(smudge),
              }),
              Transform::from_xyz(
                ((j as f32) - 2.0) * 0.2 - ((i % 2) as f32 * 0.1),
                ((i as f32) - 9.5) * 0.05,
                0.05,
              ),
            ))
          }
          rows.push(row);
        }
      }
      Self::Corner => {
        for i in 0..20 {
          let mut row = Vec::new();
          for j in 0..6 {
            let brick = Brick {
              scale: glam::Vec3::splat(smudge),
            };
            let transform = if j < 3 {
              Transform::from_xyz(
                ((j as f32) - 2.0) * 0.2 - ((i % 2) as f32 * 0.1),
                ((i as f32) - 9.5) * 0.05,
                0.05,
              )
            } else {
              let rotation = glam::Quat::from_rotation_y(PI / 2.0);
              Transform::from_xyz(
                0.05,
                ((i as f32) - 9.5) * 0.05,
                -((j as f32) - 3.0) * 0.2 + ((i % 2) as f32 * 0.1)
                  - (if j == 3 { 0.1 } else { 0.0 }),
              )
              .with_rotation(rotation)
            };
            row.push(RenderedPrimitive::new(Box::new(brick), transform));
          }
          rows.push(row);
        }
      }
    }

    RenderedFragment::new(rows.into_iter().flatten().collect())
  }
}
