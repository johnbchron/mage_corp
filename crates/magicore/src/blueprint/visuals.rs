use std::f32::consts::PI;

use bevy::{pbr::NotShadowCaster, prelude::*};
use common::materials::{ToonExtension, ToonMaterial};

use super::*;

#[derive(Component, Default, Reflect)]
struct BlueprintVisualsChild;

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

#[derive(Bundle)]
struct BlueprintVisualsBundle<M: Material> {
  spatial:           SpatialBundle,
  mesh:              Handle<Mesh>,
  material:          Handle<M>,
  marker:            BlueprintVisualsChild,
  not_shadow_caster: NotShadowCaster,
}

fn maintain_blueprint_visuals(
  time: Res<Time>,
  mut commands: Commands,
  mut materials: ResMut<Assets<ToonMaterial>>,
  prefabs: Res<BlueprintVisualsPrefabs>,
  bluep_q: Query<
    (&ActiveBlueprint, Entity, Option<&Children>),
    With<Transform>,
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
        BlueprintState::Initialized { stored }
        | BlueprintState::Built { stored } => match bluep.descriptor {
          BlueprintDescriptor::MassBarrier { radius, .. } => {
            let material = materials.add(ToonMaterial {
              base:      StandardMaterial {
                base_color: Color::rgb(0.01, 0.45, 0.81),
                ..default()
              },
              extension: ToonExtension::default(),
            });
            for i in 0..3 {
              let t = (stored / bluep.descriptor.static_cost()).clamp(0.0, 1.0);
              let t = f32::cbrt(t);

              let rot = Quat::from_euler(
                EulerRot::XYZ,
                time.elapsed_seconds() * (i as f32) * PI / 20.0,
                time.elapsed_seconds() * (((i + 1) % 3) as f32) * PI / 20.0
                  * 2.0,
                time.elapsed_seconds() * (((i + 2) % 3) as f32) * PI / 20.0
                  * 3.0,
              );
              let rot = Quat::slerp(rot, Quat::IDENTITY, t);

              p.spawn(BlueprintVisualsBundle {
                spatial:           SpatialBundle::from_transform(
                  Transform::from_rotation(rot)
                    .with_scale(Vec3::splat((3.0 - i as f32) / 3.0 * radius)),
                ),
                mesh:              prefabs.mass_barrier_mesh.clone(),
                material:          material.clone(),
                marker:            BlueprintVisualsChild,
                not_shadow_caster: NotShadowCaster,
              });
            }
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
      .register_type::<BlueprintVisualsPrefabs>()
      .init_resource::<BlueprintVisualsPrefabs>()
      .add_systems(
        Update,
        (
          maintain_blueprint_visuals,
          // animate_blueprint_visuals
        ),
      );
  }
}
