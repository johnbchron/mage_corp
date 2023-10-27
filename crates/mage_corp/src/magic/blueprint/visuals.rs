use std::f32::consts::PI;

use bevy::{pbr::NotShadowCaster, prelude::*};

use super::*;
use crate::materials::force::ForceMaterial;

#[derive(Component, Default, Reflect)]
struct BlueprintVisualsChild;

#[derive(Component, Default, Reflect)]
struct AnimationInstanceIndex(usize);

#[derive(Resource, Reflect)]
struct BlueprintVisualsPrefabs {
  mass_barrier_mesh: Handle<Mesh>,
}

impl FromWorld for BlueprintVisualsPrefabs {
  fn from_world(world: &mut World) -> Self {
    Self {
      mass_barrier_mesh: world
        .resource::<AssetServer>()
        .load("models/icosahedron.stl#wireframe"),
    }
  }
}

#[derive(Bundle, Default)]
struct BlueprintVisualsBundle<M: Material> {
  spatial:           SpatialBundle,
  mesh:              Handle<Mesh>,
  material:          Handle<M>,
  marker:            BlueprintVisualsChild,
  index:             AnimationInstanceIndex,
  not_shadow_caster: NotShadowCaster,
}

fn maintain_blueprint_visuals(
  mut commands: Commands,
  mut force_materials: ResMut<Assets<ForceMaterial>>,
  prefabs: Res<BlueprintVisualsPrefabs>,
  bluep_q: Query<
    (&ActiveBlueprint, Entity, Option<&Children>),
    Changed<ActiveBlueprint>,
  >,
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
        BlueprintStage::Initialized { stored }
        | BlueprintStage::Built { stored } => match bluep.descriptor {
          BlueprintDescriptor::MassBarrier => {
            let material = force_materials.add(ForceMaterial {
              color: Color::rgb(0.01, 0.45, 0.81),
              alpha_min: 0.0005,
              influence: 10.0,
              ..default()
            });
            for i in 0..3 {
              p.spawn(BlueprintVisualsBundle {
                spatial: SpatialBundle::from_transform(Transform::from_scale(
                  Vec3::splat((3.0 - i as f32) / 3.0),
                )),
                mesh: prefabs.mass_barrier_mesh.clone(),
                material: material.clone(),
                index: AnimationInstanceIndex(i),
                ..default()
              });
            }
          }
        },
        _ => {}
      });
  }
}

fn animate_blueprint_visuals(
  bluep_q: Query<(&ActiveBlueprint, &Children)>,
  mut visuals_q: Query<
    (&mut Transform, &AnimationInstanceIndex),
    With<BlueprintVisualsChild>,
  >,
  time: Res<Time>,
) {
  for (bluep, children) in bluep_q.iter() {
    for child in children {
      if let Ok((mut transform, index)) = visuals_q.get_mut(*child) {
        let index = index.0;
        match bluep.stage {
          BlueprintStage::Initialized { stored }
          | BlueprintStage::Built { stored } => match bluep.descriptor {
            BlueprintDescriptor::MassBarrier => {
              let t =
                (stored / bluep.descriptor.initial_cost()).clamp(0.0, 1.0);

              let rot = Quat::from_euler(
                EulerRot::XYZ,
                time.elapsed_seconds() * (index as f32) * PI / 20.0,
                time.elapsed_seconds() * (((index + 1) % 3) as f32) * PI / 20.0
                  * 2.0,
                time.elapsed_seconds() * (((index + 2) % 3) as f32) * PI / 20.0
                  * 3.0,
              );
              transform.rotation = Quat::slerp(rot, Quat::IDENTITY, t);
            }
          },
          _ => {}
        }
      }
    }
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
      .add_systems(
        Update,
        (maintain_blueprint_visuals, animate_blueprint_visuals),
      );
  }
}
