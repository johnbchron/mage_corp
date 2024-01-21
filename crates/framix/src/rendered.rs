use bevy::prelude::*;
use bevy_implicits::{
  prelude::{ImplicitInputs, MesherDetail, MesherInputs, MesherRegion},
  SyncImplicitsOnce,
};
use bevy_xpbd_3d::components::RigidBody;
use common::materials::ToonMaterial;

use crate::RenderedPrimitive;

#[derive(Reflect)]
pub struct RenderedModule {
  primitives: Vec<RenderedPrimitive>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct RenderedModuleMarker;

impl RenderedModule {
  pub fn new(primitives: Vec<RenderedPrimitive>) -> Self { Self { primitives } }

  pub fn spawn(
    &self,
    transform: Transform,
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

    commands
      .spawn((
        SpatialBundle::from_transform(transform),
        RenderedModuleMarker,
      ))
      .with_children(|parent| {
        self.primitives.iter().enumerate().for_each(|(i, p)| {
          let collider_attempt = p.primitive.collider();
          let aabb = p.primitive.aabb();

          let mut entity = parent.spawn((
            SpatialBundle::from_transform(p.transform),
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
              gen_collider: collider_attempt.is_none(),
            }),
            SyncImplicitsOnce,
            RigidBody::Static,
            p.primitive.density(),
            p.primitive.friction(),
            p.primitive.restitution(),
            Name::new(format!("building_primitive_{}", i)),
          ));
          if let Some(collider) = collider_attempt {
            entity.insert(collider);
          }
        });
      });
  }
}

pub struct RenderedModulePlugin;

impl Plugin for RenderedModulePlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<RenderedModuleMarker>()
      .add_systems(Update, render_module_debug_cubes);
  }
}

fn render_module_debug_cubes(
  mut gizmos: Gizmos,
  q: Query<&Transform, With<RenderedModuleMarker>>,
) {
  for transform in q.iter() {
    gizmos.cuboid(transform.clone(), Color::WHITE);
  }
}
