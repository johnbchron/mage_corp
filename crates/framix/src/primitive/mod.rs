//! Physical definitions of building primitives.

pub mod brick;
pub mod plank;

use bevy::{
  pbr::StandardMaterial,
  reflect::Reflect,
  render::{color::Color, primitives::Aabb},
};
use bevy_implicits::prelude::{builder as sb, Shape};
use bevy_xpbd_3d::components::{
  Collider, ColliderDensity, Friction, Restitution,
};
use common::materials::{ToonExtension, ToonMaterial};

pub use self::{brick::Brick, plank::Plank};

/// A trait for physical definitions of a physical building primitive.
pub trait Primitive: Reflect + Send + Sync + 'static {
  /// The [`AABB`] of the primitive.
  fn aabb(&self) -> Aabb;
  /// The [`Shape`] of the primitive.
  fn shape(&self) -> Shape;
  /// The [`Shape`] to use for calculating the primitive's [Collider], in the
  /// case that `collider_shape` returns `None`. This method is useful if a
  /// convex decomposition is needed, but internal features exist in the
  /// `shape` that are unnecessary for the collider.
  fn collider_shape(&self) -> Shape { self.shape() }
  /// The [`Collider`] of the primitive, if it's feasible to generate directly.
  /// Otherwise return `None` and the `collider_shape` will be used in a convex
  /// decomposition to generate the collider.
  fn collider(&self) -> Option<Collider> { None }
  /// The resolution at which to tessellate the primitive, in cells per meter.
  fn resolution(&self) -> f32 { 200.0 }
  /// The [`ToonMaterial`] of the primitive. This will be deduplicated by
  /// [`RenderedModule::spawn`].
  fn material(&self) -> ToonMaterial;
  /// The density properties of the primitive.
  fn density(&self) -> ColliderDensity;
  /// The friction properties of the primitive.
  fn friction(&self) -> Friction;
  /// The restitution properties of the primitive.
  fn restitution(&self) -> Restitution;
}
