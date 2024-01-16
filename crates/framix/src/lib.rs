#![warn(missing_docs)]
#![feature(trivial_bounds)]

//! Framix is a crate for procedurally generating buildings.
//!
//! The primary API is composed of the [`Module`] trait and the [`Primitive`]
//! trait.
//!
//! The [`Module`] trait is intended to be implemented by users on marker types.
//! The marker types can then be laid out by a user's algorithm to form a
//! building. The trait has a [`render`] method, which returns a
//! [`RenderedModule`] that can be used to spawn the building chunk
//! into the game world.
//!
//! The [`Primitive`] trait defines primitives that can be used to construct
//! modules. The trait has a number of methods that define the properties of the
//! primitive, such as its [`Shape`](bevy_implicits::prelude::Shape),
//! [`Collider`](bevy_xpbd_3d::components::Collider),
//! [`ToonMaterial`](common::materials::ToonMaterial), etc.
//!
//! Essentially, implement [`Primitive`] on the physical building blocks of your
//! building (such as wood planks, shingles, etc.), and implement [`Module`] on
//! the semantic building blocks of your building, (such as a brick wall or
//! roof). The [`Module`] types configure and arrange primitives which can then
//! be spawned into the world.

pub mod primitive;
mod rendered;
pub mod prelude {
  //! A collection of commonly used types and traits.
  pub use crate::{primitive::Primitive, Module, RenderedPrimitive};
}

use bevy::prelude::*;

use self::{
  primitive::{Brick, Primitive},
  rendered::RenderedModule,
};

/// A rendered [`Primitive`].
#[derive(Reflect)]
pub struct RenderedPrimitive {
  primitive: Box<dyn Primitive>,
  transform: Transform,
}

impl RenderedPrimitive {
  /// Create a new [`RenderedPrimitive`].
  pub fn new(primitive: Box<dyn Primitive>, transform: Transform) -> Self {
    Self {
      primitive,
      transform,
    }
  }
}

/// A trait for semantic definitions of a building chunk.
///
/// This trait is intended to be implemented by users on marker types, which
/// can be laid out by an algorithm to form a building. The trait has a
/// [`render`] method, which returns a [`RenderedModule`] that can be used to
/// spawn the building chunk.
///
/// A module should be one meter cubed, on the interval [-0.5, 0.5] in each
/// dimension. While a module can be larger than this, its over-reach and
/// under-reach should be semetrical, similar to the interlocking bricks in a
/// brick wall.
pub trait Module {
  /// Render the module.
  ///
  /// This method returns a [`RenderedModule`] that can be used to spawn the
  /// building chunk into the game world.
  fn render(&self) -> RenderedModule;
}

/// A brick wall module.
pub struct BrickWall;

impl Module for BrickWall {
  fn render(&self) -> RenderedModule {
    // we'll fit 5 bricks end to end and 20 stacked. we'll also offset every
    // other brick by one half length.
    let smudge = 1.02;

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
            0.0,
          ),
        })
      }
      rows.push(row);
    }

    RenderedModule::new(rows.into_iter().flatten().collect())
  }
}
