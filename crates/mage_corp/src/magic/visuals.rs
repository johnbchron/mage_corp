use bevy::{pbr::wireframe::Wireframe, prelude::*};

use super::*;
use crate::{materials::toon::ToonMaterial, utils::despawn::DespawnTag};

#[derive(Component, Default)]
pub struct BlueprintVisualsChild;

#[derive(Resource)]
pub struct BlueprintVisualsPrefabs {
  mass_barrier_mesh:     Handle<Mesh>,
  mass_barrier_material: Handle<ToonMaterial>,
}

impl FromWorld for BlueprintVisualsPrefabs {
  fn from_world(world: &mut World) -> Self {
    Self {
      mass_barrier_mesh:     world.resource_mut::<Assets<Mesh>>().add(
        Mesh::try_from(shape::Icosphere {
          radius:       1.0,
          subdivisions: 1,
        })
        .unwrap(),
      ),
      mass_barrier_material: world.resource_mut::<Assets<ToonMaterial>>().add(
        ToonMaterial {
          color: Color::rgb(0.01, 0.45, 0.81),
          ..default()
        },
      ),
    }
  }
}

#[derive(Bundle, Default)]
struct BlueprintVisualsBundle<M: Material> {
  spatial:  SpatialBundle,
  mesh:     Handle<Mesh>,
  material: Handle<M>,
  marker:   BlueprintVisualsChild,
}

pub fn maintain_blueprint_visuals(
  mut commands: Commands,
  prefabs: Res<BlueprintVisualsPrefabs>,
  bluep_q: Query<(&Blueprint, Entity, Option<&Children>), Changed<Blueprint>>,
  visuals_q: Query<Entity, With<BlueprintVisualsChild>>,
) {
  for (bluep, entity, children) in &bluep_q {
    // if the entity already has children
    if let Some(children) = children {
      let visuals_children = children
        .iter()
        .filter_map(|c| visuals_q.get(*c).ok())
        .collect::<Vec<_>>();

      // this unwrap is permissible because bevy agrees to never supply an
      // empty children component.
      let child = visuals_children.first().unwrap();
      commands
        .entity(*child)
        .insert(prefabs.mass_barrier_mesh.clone())
        .insert(prefabs.mass_barrier_material.clone());

      // if there's more than one VisualsChild
      if visuals_children.len() > 1 {
        visuals_children.into_iter().skip(1).for_each(|e| {
          commands.entity(e).insert(DespawnTag);
        });
      }
    } else {
      commands.entity(entity).with_children(|p| {
        p.spawn(BlueprintVisualsBundle {
          mesh: prefabs.mass_barrier_mesh.clone(),
          material: prefabs.mass_barrier_material.clone(),
          ..default()
        });
      });
    }
  }
}
