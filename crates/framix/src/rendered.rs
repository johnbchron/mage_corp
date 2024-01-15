use bevy::prelude::*;
use bevy_implicits::{
  prelude::{ImplicitInputs, MesherDetail, MesherInputs, MesherRegion},
  SyncImplicits,
};
use bevy_xpbd_3d::components::RigidBody;
use common::materials::ToonMaterial;

use crate::PositionedPrimitive;

#[derive(Reflect)]
pub struct RenderedModule {
  primitives: Vec<PositionedPrimitive>,
}

impl RenderedModule {
  pub fn new(primitives: Vec<PositionedPrimitive>) -> Self {
    Self { primitives }
  }

  pub fn spawn(
    &self,
    translation: glam::Vec3,
    commands: &mut Commands,
    toon_materials: &mut ResMut<Assets<ToonMaterial>>,
  ) {
    println!(
      "spawning rendered module with {} primitives",
      self.primitives.len()
    );
    // all un-deduplicated materials used by primitves
    let primitive_materials = self
      .primitives
      .iter()
      .enumerate()
      .map(|(i, p)| (i, p.primitive.material()))
      .collect::<Vec<_>>();
    let deduped_materials = {
      let mut materials = primitive_materials.clone();
      materials.dedup_by(|(_, a), (_, b)| a.reflect_partial_eq(b).unwrap());
      materials.into_iter().map(|e| e.1).collect::<Vec<_>>()
    };
    // a map from primitive index to deduplicated material
    let material_map = primitive_materials
      .into_iter()
      .map(|(_, m)| {
        for (j, dm) in deduped_materials.iter().enumerate() {
          if m.reflect_partial_eq(dm).unwrap() {
            return j;
          }
        }
        panic!("failed to find material in deduplicated list");
      })
      .collect::<Vec<_>>();
    let material_handles = deduped_materials
      .into_iter()
      .map(|m| toon_materials.add(m))
      .collect::<Vec<_>>();

    self.primitives.iter().enumerate().for_each(|(i, p)| {
      let collider_attempt = p.primitive.collider();
      let aabb = p.primitive.aabb();
      let mut transform = p.transform.clone();
      transform.translation += translation;
      let mut entity = commands.spawn((
        SpatialBundle::from_transform(transform),
        material_handles[material_map[i]].clone(),
        ImplicitInputs(MesherInputs {
          shape:        p.primitive.shape(),
          region:       MesherRegion {
            position: aabb.center,
            scale:    aabb.half_extents * 2.0,
            detail:   MesherDetail::Resolution(200.0),
            prune:    false,
            simplify: false,
          },
          gen_collider: !collider_attempt.is_some(),
        }),
        SyncImplicits,
        RigidBody::Static,
        p.primitive.density(),
        p.primitive.friction(),
        p.primitive.restitution(),
      ));
      if let Some(collider) = collider_attempt {
        entity.insert(collider);
      }
    });
  }
}
