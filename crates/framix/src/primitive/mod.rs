//! Physical definitions of building primitives.

pub mod brick;
pub mod concrete;
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

pub use self::{brick::Brick, concrete::ConcreteBlock, plank::Plank};
use crate::{find_or_add::FindOrAdd, spawnable::Spawnable};

/// A trait for physical definitions of a physical building primitive.
pub trait Primitive: Spawnable<SpawnContext = (Entity, Transform)> {
  /// The [`Aabb`] of the primitive.
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
  /// The [`ToonMaterial`] of the primitive.
  fn material(&self) -> ToonMaterial;
  /// The density properties of the primitive.
  fn density(&self) -> ColliderDensity;
  /// The friction properties of the primitive.
  fn friction(&self) -> Friction;
  /// The restitution properties of the primitive.
  fn restitution(&self) -> Restitution;
}

impl<T: Primitive> Spawnable for T {
  // the spawn context here is the entity's parent and transform.
  type SpawnContext = (Entity, Transform);

  /// Spawn the primitive into the world under the given parent.
  fn spawn(&self, world: &mut World, (parent, transform): Self::SpawnContext) {
    let material_handle = {
      world
        .resource_mut::<Assets<ToonMaterial>>()
        .find_or_add(self.material())
    };

    let collider_attempt = self.collider();
    let aabb = self.aabb();

    world.entity_mut(parent).with_children(|p| {
      let mut entity = p.spawn((
        SpatialBundle::from_transform(transform),
        material_handle,
        ImplicitInputs(MesherInputs {
          shape:        self.shape(),
          region:       MesherRegion {
            position: aabb.center,
            scale:    aabb.half_extents * 2.0,
            detail:   MesherDetail::Resolution(self.resolution()),
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
    });
  }
}
