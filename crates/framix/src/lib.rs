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

pub mod brick_wall;
mod find_or_add;
pub mod primitive;
mod rendered;

use bevy::{prelude::*, utils::HashMap};
use common::materials::ToonMaterial;

pub use self::{brick_wall::*, rendered::RenderedModulePlugin};
use self::{primitive::Brick, rendered::RenderedModule};
pub use crate::primitive::Primitive;

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

impl RenderedPrimitive {
  fn spawn(
    &self,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ToonMaterial>,
  ) {
    self.primitive.spawn(parent, materials, self.transform);
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

/// A composition of modules used to construct a building.
pub struct Composition {
  modules: HashMap<IVec3, Box<dyn Module>>,
}

impl Composition {
  /// Creates a new [`Composition`].
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
    }
  }

  /// Adds a module to the composition.
  pub fn add_module(&mut self, module: impl Module + 'static, position: IVec3) {
    self.modules.insert(position, Box::new(module));
  }

  /// Spawns the composition into the world.
  pub fn spawn(
    &self,
    commands: &mut Commands,
    materials: &mut Assets<ToonMaterial>,
  ) {
    commands
      .spawn((SpatialBundle::default(), Name::new("building_composition")))
      .with_children(|p| {
        for (position, module) in self.modules.iter() {
          let transform = Transform::from_translation(Vec3::new(
            position.x as f32,
            position.y as f32,
            position.z as f32,
          ));
          module.render().spawn(p, materials, transform);
        }
      });
  }
}

impl Default for Composition {
  fn default() -> Self { Self::new() }
}
