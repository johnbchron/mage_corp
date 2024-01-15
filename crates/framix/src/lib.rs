// #![warn(missing_docs)]

mod primitive;

use bevy::prelude::*;

use self::primitive::{Brick, Primitive};

pub struct PositionedPrimitive {
  primitive: Box<dyn Primitive>,
  transform: Transform,
}

pub mod rendered;

use rendered::RenderedModule;

pub trait Module {
  fn render(&self) -> RenderedModule;
}

pub struct BrickWall;

impl Module for BrickWall {
  fn render(&self) -> RenderedModule {
    // we'll fit 5 bricks end to end and 20 stacked. we'll also offset every
    // other brick by one half length.
    let smudge = 1.01;

    let mut rows = Vec::new();
    for i in 0..20 {
      let mut row = Vec::new();
      for j in 0..5 {
        row.push(PositionedPrimitive {
          primitive: Box::new(Brick {
            scale: glam::Vec3::splat(smudge),
          }),
          transform: Transform::from_xyz(
            ((j as f32) - 2.0) * 0.2 - ((i % 2) as f32 * 0.1),
            ((i as f32) - 9.5) * 0.05,
            0.0,
          ),
        })
      }
      rows.push(row);
    }

    RenderedModule::new(rows.into_iter().flatten().collect())
  }
}
