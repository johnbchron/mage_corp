//! Physical definitions of building primitives.

pub mod brick;
pub mod plank;

use bevy::{
  prelude::{Transform, *},
  render::primitives::Aabb,
};
use bevy_implicits::{
  prelude::{builder as sb, *},
  SyncImplicitsOnce,
};
use bevy_xpbd_3d::components::{
  Collider, ColliderDensity, Friction, Restitution, RigidBody,
};
use common::materials::{ToonExtension, ToonMaterial};

pub use self::{brick::Brick, plank::Plank};
use crate::find_or_add::FindOrAdd;

/// A trait for physical definitions of a physical building primitive.
pub trait Primitive {
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

  /// Spawn the primitive into the world under the given parent.
  fn spawn(
    &self,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ToonMaterial>,
    transform: Transform,
  ) {
    let material_handle = materials.find_or_add(self.material());

    let collider_attempt = self.collider();
    let aabb = self.aabb();

    let mut entity = parent.spawn((
      SpatialBundle::from_transform(transform),
      material_handle,
      ImplicitInputs(MesherInputs {
        shape:        self.shape(),
        region:       MesherRegion {
          position: aabb.center,
          scale:    aabb.half_extents * 2.0,
          detail:   MesherDetail::Resolution(200.0),
          prune:    false,
          simplify: false,
        },
        gen_collider: collider_attempt.is_none(),
      }),
      SyncImplicitsOnce,
      RigidBody::Static,
      self.density(),
      self.friction(),
      self.restitution(),
      Name::new("building_primitive"),
    ));
    if let Some(collider) = collider_attempt {
      entity.insert(collider);
    }
  }
}
