// #![warn(missing_docs)]
#![feature(trivial_bounds)]

//! Framix is a crate for procedurally generating buildings.
//!
//! The primary API is composed of the [`FragmentConfig`] trait and the
//! [`Primitive`] trait.
//!
//! The [`FragmentConfig`] trait is intended to be implemented by users on
//! marker types that semantically represent a piece of a building, like
//! `BrickWallFragment`. The marker types can then be laid out by a user's
//! algorithm to form a building. The trait has a `render` method, which
//! returns a `RenderedFragment` that can be used to spawn the building chunk
//! into the game world. Make sure when adding fragments to add a variant within
//! the `Fragment` enum.
//!
//! The [`Primitive`] trait defines primitives that can be used to populate
//! fragments. The trait has a number of methods that define the properties of
//! the primitive, such as its [`Shape`](bevy_implicits::prelude::Shape),
//! [`Collider`](bevy_xpbd_3d::components::Collider),
//! [`ToonMaterial`], etc.
//!
//! Essentially, implement [`Primitive`] on the physical building blocks of your
//! building (such as wood planks, shingles, etc.), and implement
//! [`FragmentConfig`] on the semantic building blocks of your building, (such
//! as a brick wall or roof). The [`FragmentConfig`] types configure and arrange
//! primitives which can then be spawned into the world.

pub mod brick_wall;
mod find_or_add;
pub mod foundation;
pub mod primitive;
mod rendered;

use bevy::{prelude::*, utils::HashMap};
use common::materials::ToonMaterial;

pub use self::{brick_wall::*, foundation::*, rendered::FragmentDebugPlugin};
use self::{
  primitive::Brick,
  rendered::{RenderedFragment, RenderedFragmentMarker},
};
pub use crate::primitive::Primitive;

#[derive(Reflect)]
pub enum Fragment {
  BrickWall(BrickWallFragment),
  Foundation(FoundationFragment),
}

impl Fragment {
  pub fn render(&self) -> RenderedFragment {
    match self {
      Self::BrickWall(fragment) => fragment.render(),
      Self::Foundation(fragment) => fragment.render(),
    }
  }
}

pub trait FragmentConfig {
  fn render(&self) -> RenderedFragment;
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

/// The coordinates of a fragment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
pub struct FragmentCoords {
  pub position:  IVec3,
  pub direction: Direction,
}

impl FragmentCoords {
  /// Creates a new [`FragmentCoords`].
  pub fn new(position: IVec3, direction: Direction) -> Self {
    Self {
      position,
      direction,
    }
  }
  /// Creates a new [`FragmentCoords`] with the default direction.
  pub fn new_default_dir(position: IVec3) -> Self {
    Self::new(position, Direction::default())
  }
}

impl From<IVec3> for FragmentCoords {
  fn from(position: IVec3) -> Self { Self::new_default_dir(position) }
}

impl From<FragmentCoords> for Transform {
  fn from(coords: FragmentCoords) -> Self {
    let mut transform = Transform::from_translation(Vec3::new(
      coords.position.x as f32,
      coords.position.y as f32,
      coords.position.z as f32,
    ));
    transform.rotate(Quat::from_rotation_y(coords.direction.to_rotation()));
    transform
  }
}

/// A composition of fragments used to construct a building.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Composition {
  fragments: HashMap<FragmentCoords, Fragment>,
}

impl Composition {
  /// Creates a new [`Composition`].
  pub fn new() -> Self {
    Self {
      fragments: HashMap::new(),
    }
  }

  /// Adds a fragment to the composition.
  pub fn add_fragment(&mut self, fragment: Fragment, coords: FragmentCoords) {
    self.fragments.insert(coords, fragment);
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
        for (coords, fragment) in self.fragments.iter() {
          fragment.render().spawn(p, materials, (*coords).into());
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
    app.register_type::<RenderedFragmentMarker>();
    app.register_type::<Composition>();
  }
}
