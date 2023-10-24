use bevy::{pbr::NotShadowCaster, prelude::*};

use super::*;
use crate::materials::force::ForceMaterial;

#[derive(Component, Default, Reflect)]
struct BlueprintVisualsChild;

#[derive(Component, Default, Reflect)]
struct AnimationInstanceIndex(usize);

#[derive(Resource, Reflect)]
struct BlueprintVisualsPrefabs {
  mass_barrier_mesh:     Handle<Mesh>,
  mass_barrier_material: Handle<ForceMaterial>,
}

impl FromWorld for BlueprintVisualsPrefabs {
  fn from_world(world: &mut World) -> Self {
    Self {
      mass_barrier_mesh:     world
        .resource::<AssetServer>()
        .load("models/icosahedron.stl#wireframe"),
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

fn maintain_blueprint_visuals(
  mut commands: Commands,
  prefabs: Res<BlueprintVisualsPrefabs>,
  bluep_q: Query<(&Blueprint, Entity, Option<&Children>), Changed<Blueprint>>,
  visuals_q: Query<Entity, With<BlueprintVisualsChild>>,
) {
  for (bluep, entity, children) in &bluep_q {
    // if the entity already has `BlueprintVisualsChild` children, delete them
    if let Some(children) = children {
      children
        .iter()
        .filter_map(|c| visuals_q.get(*c).ok())
        .for_each(|e| {
          commands.entity(e).despawn_recursive();
        });
    }

    // spawn in the new child
    commands
      .entity(entity)
      .with_children(|p| match bluep.stage {
        BlueprintStage::Initialized => match bluep._type {
          BlueprintType::MassBarrier => {
            p.spawn(BlueprintVisualsBundle {
              mesh: prefabs.mass_barrier_mesh.clone(),
              material: prefabs.mass_barrier_material.clone(),
              ..default()
            });
          }
        },
        _ => {}
      });
  }
}

pub struct BlueprintVisualsPlugin;

impl Plugin for BlueprintVisualsPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<BlueprintVisualsChild>()
      .register_type::<AnimationInstanceIndex>()
      .register_type::<BlueprintVisualsPrefabs>()
      .init_resource::<BlueprintVisualsPrefabs>()
      .add_systems(Update, maintain_blueprint_visuals);
  }
}
