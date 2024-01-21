//! Brick wall modules.

use std::f32::consts::PI;

use super::*;

/// A brick wall module.
pub struct BrickWall;

impl Module for BrickWall {
  fn render(&self) -> RenderedModule {
    // we'll fit 5 bricks end to end and 20 stacked. we'll also offset every
    // other brick by one half length.
    let smudge = 1.02;

    // honestly, this is barely comprehensible. it's almost not worth debugging.
    // if you encounter an issue, consider rewriting.
    let mut rows = Vec::new();
    for i in 0..20 {
      let mut row = Vec::new();
      for j in 0..5 {
        row.push(RenderedPrimitive {
          primitive: Box::new(Brick {
            scale: glam::Vec3::splat(smudge),
          }),
          transform: Transform::from_xyz(
            ((j as f32) - 2.0) * 0.2 - ((i % 2) as f32 * 0.1),
            ((i as f32) - 9.5) * 0.05,
            0.05,
          ),
        })
      }
      rows.push(row);
    }

    RenderedModule::new(rows.into_iter().flatten().collect())
  }
}

/// A brick corner wall module.
pub struct BrickCornerWall;

impl Module for BrickCornerWall {
  fn render(&self) -> RenderedModule {
    let smudge = 1.02;

    // this is so arcane, I don't even know what to say. I just know that it
    // works. do not touch.
    let mut rows = Vec::new();
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
        row.push(RenderedPrimitive {
          primitive: Box::new(brick),
          transform,
        });
      }
      rows.push(row);
    }

    RenderedModule::new(rows.into_iter().flatten().collect())
  }
}
