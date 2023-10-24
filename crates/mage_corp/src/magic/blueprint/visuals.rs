use bevy::{pbr::NotShadowCaster, prelude::*};

use super::*;
use crate::materials::force::ForceMaterial;

#[derive(Component, Default)]
pub struct BlueprintVisualsChild;

#[derive(Resource)]
pub struct BlueprintVisualsPrefabs {
  mass_barrier_mesh:     Handle<Mesh>,
  mass_barrier_material: Handle<ForceMaterial>,
}

impl FromWorld for BlueprintVisualsPrefabs {
  fn from_world(world: &mut World) -> Self {
    Self {
      mass_barrier_mesh:     world
        .resource::<AssetServer>()
        .load("models/icosahedron.stl"),
      mass_barrier_material: world
        .resource_mut::<Assets<ForceMaterial>>()
        .add(ForceMaterial::from(Color::rgb(0.01, 0.45, 0.81))),
    }
  }
}

#[derive(Bundle, Default)]
struct BlueprintVisualsBundle<M: Material> {
  spatial:           SpatialBundle,
  mesh:              Handle<Mesh>,
  material:          Handle<M>,
  marker:            BlueprintVisualsChild,
  not_shadow_caster: NotShadowCaster,
}

pub fn maintain_blueprint_visuals(
  mut commands: Commands,
  prefabs: Res<BlueprintVisualsPrefabs>,
  bluep_q: Query<(&Blueprint, Entity, Option<&Children>), Changed<Blueprint>>,
  visuals_q: Query<Entity, With<BlueprintVisualsChild>>,
) {
  for (_bluep, entity, children) in &bluep_q {
    // if the entity already has children, delete them all
    if let Some(children) = children {
      children
        .iter()
        .filter_map(|c| visuals_q.get(*c).ok())
        .for_each(|e| {
          commands.entity(e).despawn_recursive();
        });
    }

    // spawn in the new child
    commands.entity(entity).with_children(|p| {
      p.spawn(BlueprintVisualsBundle {
        mesh: prefabs.mass_barrier_mesh.clone(),
        material: prefabs.mass_barrier_material.clone(),
        ..default()
      });
    });
  }
}
