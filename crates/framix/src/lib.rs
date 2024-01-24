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
pub mod foundation;
pub mod primitive;
mod rendered;

use bevy::{prelude::*, utils::HashMap};
use common::materials::ToonMaterial;

pub use self::{brick_wall::*, foundation::*, rendered::RenderedModulePlugin};
use self::{
  primitive::Brick,
  rendered::{RenderedModule, RenderedModuleMarker},
};
pub use crate::primitive::Primitive;

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
pub trait Module: Reflect {
  /// Render the module.
  ///
  /// This method returns a [`RenderedModule`] that can be used to spawn the
  /// building chunk into the game world.
  fn render(&self) -> RenderedModule;
}

/// A 2d direction.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Reflect)]
pub enum Direction {
  North,
  East,
  #[default]
  South,
  West,
}

impl Direction {
  /// Returns the rotation of the direction.
  pub fn to_rotation(self) -> f32 {
    match self {
      Self::North => 0.0,
      Self::East => -std::f32::consts::FRAC_PI_2,
      Self::South => -std::f32::consts::PI,
      Self::West => -std::f32::consts::FRAC_PI_2 * 3.0,
    }
  }
}

/// The coordinates of a module.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct ModuleCoords {
  pub position:  IVec3,
  pub direction: Direction,
}

impl ModuleCoords {
  /// Creates a new [`ModuleCoords`].
  pub fn new(position: IVec3, direction: Direction) -> Self {
    Self {
      position,
      direction,
    }
  }
  /// Creates a new [`ModuleCoords`] with the default direction.
  pub fn new_default_dir(position: IVec3) -> Self {
    Self::new(position, Direction::default())
  }
}

impl From<IVec3> for ModuleCoords {
  fn from(position: IVec3) -> Self { Self::new_default_dir(position) }
}

impl From<ModuleCoords> for Transform {
  fn from(coords: ModuleCoords) -> Self {
    let mut transform = Transform::from_translation(Vec3::new(
      coords.position.x as f32,
      coords.position.y as f32,
      coords.position.z as f32,
    ));
    transform.rotate(Quat::from_rotation_y(coords.direction.to_rotation()));
    transform
  }
}

/// A composition of modules used to construct a building.
#[derive(Component, Default, Reflect)]
#[reflect(from_reflect = false)]
pub struct Composition {
  modules: HashMap<ModuleCoords, Box<dyn Module + Send + Sync + 'static>>,
}

impl Composition {
  /// Creates a new [`Composition`].
  pub fn new() -> Self {
    Self {
      modules: HashMap::new(),
    }
  }

  /// Adds a module to the composition.
  pub fn add_module(
    &mut self,
    module: impl Module + Send + Sync + 'static,
    coords: ModuleCoords,
  ) {
    self.modules.insert(coords, Box::new(module));
  }

  /// Spawns the composition into the world.
  pub fn spawn(
    self,
    transform: Transform,
    commands: &mut Commands,
    materials: &mut Assets<ToonMaterial>,
  ) -> Entity {
    commands
      .spawn((
        SpatialBundle::from_transform(transform),
        Name::new("building_composition"),
      ))
      .with_children(|p| {
        for (coords, module) in self.modules.iter() {
          module.render().spawn(p, materials, (*coords).into());
        }
      })
      .insert(self)
      .id()
  }
}

/// The `framix` plugin.
///
/// This plugin mainly registers types.
pub struct FramixPlugin;

impl Plugin for FramixPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<RenderedModuleMarker>();
  }
}
