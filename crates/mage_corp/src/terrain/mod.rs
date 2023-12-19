mod config;
mod regions;
mod timing;

use bevy::prelude::*;
use bevy_implicits::prelude::*;
use bevy_xpbd_3d::prelude::*;

use self::config::TerrainConfig;
use crate::materials::{ToonExtension, ToonMaterial};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TerrainDetailTarget;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TerrainCurrentShape(#[reflect(ignore)] pub Shape);

impl Default for TerrainCurrentShape {
  fn default() -> Self {
    TerrainCurrentShape(Shape::new_expr(
      "(sqrt(square(x) + square(y + 5000) + square(z)) - 5000) + ((sin(x / \
       20.0) + sin(y / 20.0) + sin(z / 20.0)) * 4.0)",
    ))
  }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainGenerations {
  pub current: (u32, Vec3),
  pub next:    Vec<(u32, Vec3)>,
}

impl TerrainGenerations {
  pub fn next(&self) -> u32 {
    u32::max(
      self.current.0 + 1,
      self.next.iter().map(|(i, _)| i).max().copied().unwrap_or(0) + 1,
    )
  }
}

#[derive(Component, Reflect)]
pub struct TerrainPiece {
  pub generation: u32,
}

#[derive(Event)]
pub struct TerrainTriggerRegeneration {
  pub target_location: Vec3,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TerrainMaterial {
  pub material: Handle<ToonMaterial>,
}

impl FromWorld for TerrainMaterial {
  fn from_world(world: &mut World) -> Self {
    let mut materials =
      world.get_resource_mut::<Assets<ToonMaterial>>().unwrap();
    let material = materials.add(ToonMaterial {
      base:      StandardMaterial {
        base_color: Color::hex("5DBB63").unwrap(),
        ..default()
      },
      extension: ToonExtension {
        outline_depth_threshold: 10.0,
        outline_normal_threshold: 10.0,
        ..default()
      },
    });
    TerrainMaterial { material }
  }
}

#[derive(Bundle)]
pub struct TerrainBundle {
  pub spatial:       SpatialBundle,
  pub implicit_mesh: Handle<ImplicitMesh>,
  pub mesh:          Handle<Mesh>,
  pub material:      Handle<ToonMaterial>,
  pub rigid_body:    RigidBody,
  pub position:      Position,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<TerrainConfig>()
      .init_resource::<TerrainCurrentShape>()
      .init_resource::<TerrainGenerations>()
      .init_resource::<TerrainMaterial>()
      .register_type::<TerrainDetailTarget>()
      .register_type::<TerrainPiece>()
      .register_type::<TerrainConfig>()
      .register_type::<TerrainCurrentShape>()
      .register_type::<TerrainGenerations>()
      .register_type::<TerrainMaterial>()
      .add_event::<TerrainTriggerRegeneration>()
      .add_systems(
        Update,
        (
          kickstart_terrain,
          graduate_generation,
          clean_generation,
          create_generation,
        )
          .chain(),
      )
      .add_plugins(timing::TerrainGenerationTimingPlugin);
  }
}

fn kickstart_terrain(
  generations: Res<TerrainGenerations>,
  config: Res<TerrainConfig>,
  shape: Res<TerrainCurrentShape>,
  mut event_writer: EventWriter<TerrainTriggerRegeneration>,
  target_q: Query<&Transform, With<TerrainDetailTarget>>,
) {
  let Ok(transform) = target_q.get_single() else {
    return;
  };
  let mut reason: Option<&str> = None;

  if generations.current.0 == 0 && generations.next.is_empty() {
    reason = Some("no generations");
  } else if config.is_changed() {
    reason = Some("config changed");
  } else if shape.is_changed() {
    reason = Some("shape changed");
  } else if config.too_far_away(generations.current.1, transform.translation)
    && generations
      .next
      .last()
      .map(|(_, pos)| *pos != transform.translation)
      .unwrap_or(true)
  {
    reason = Some("too far away");
  }

  if let Some(reason) = reason {
    info!("triggering terrain regeneration: {}", reason);
    event_writer.send(TerrainTriggerRegeneration {
      target_location: transform.translation,
    });
  }
}

fn create_generation(
  mut commands: Commands,
  mut generations: ResMut<TerrainGenerations>,
  mut event_reader: EventReader<TerrainTriggerRegeneration>,
  shape: Res<TerrainCurrentShape>,
  config: Res<TerrainConfig>,
  asset_server: Res<AssetServer>,
) {
  let Some(event) = event_reader.read().next() else {
    return;
  };

  let gen_id = generations.next();

  for (i, region) in regions::calculate_regions(&config, event.target_location)
    .into_iter()
    .enumerate()
  {
    let inputs = MesherInputs {
      shape: shape.0.clone(),
      region,
      gen_collider: true,
    };
    let path =
      bevy_implicits::asset_path(inputs).expect("failed to get mesh path");

    let handle: Handle<ImplicitMesh> = asset_server.load(path);
    commands.spawn((
      TerrainPiece { generation: gen_id },
      handle,
      Name::new(format!("terrain-{:03}-{:04}", gen_id, i)),
    ));
  }

  generations.next.push((gen_id, event.target_location));
}

fn graduate_generation(
  mut commands: Commands,
  mut generations: ResMut<TerrainGenerations>,
  terrain_material: Res<TerrainMaterial>,
  q: Query<(Entity, &TerrainPiece, &Handle<ImplicitMesh>)>,
  asset_server: Res<AssetServer>,
  implicit_meshes: Res<Assets<ImplicitMesh>>,
  colliders: Res<Assets<ColliderAsset>>,
) {
  if generations.next.is_empty() {
    return;
  }

  let q_list = q.iter().collect::<Vec<_>>();

  let mut unloaded_generations = q_list
    .clone()
    .into_iter()
    .filter_map(|(_, piece, handle)| {
      match asset_server.is_loaded_with_dependencies(handle) {
        false => Some(piece.generation),
        true => None,
      }
    })
    .collect::<Vec<_>>();

  unloaded_generations.sort();
  unloaded_generations.dedup();

  // subtract to get the loaded generations
  let mut loaded_generations = generations
    .next
    .iter()
    .copied()
    .filter(|gen| !unloaded_generations.contains(&gen.0))
    .collect::<Vec<_>>();
  loaded_generations.sort_by_key(|gen| gen.0);

  // if no loaded generations, then we can't graduate
  if loaded_generations.is_empty() {
    return;
  }

  // pick the latest loaded generation
  let latest_loaded = *loaded_generations.last().unwrap();
  generations.current = latest_loaded;
  info!("graduated to terrain generation {:?}", latest_loaded);

  // add the terrain bundle to the entities of the new generation
  for (entity, piece, handle) in q_list.into_iter() {
    if piece.generation == latest_loaded.0 {
      let implicit_mesh = implicit_meshes.get(handle).unwrap();

      commands.entity(entity).insert(TerrainBundle {
        spatial:       SpatialBundle {
          transform: Transform::from_translation(
            implicit_mesh.inputs.region.position.into(),
          ),
          ..SpatialBundle::default()
        },
        implicit_mesh: handle.clone(),
        mesh:          implicit_mesh.mesh.clone(),
        material:      terrain_material.material.clone(),
        rigid_body:    RigidBody::Static,
        position:      Position(implicit_mesh.inputs.region.position.into()),
      });

      // add the collider if it exists
      if let Some(collider) = colliders
        .get(implicit_mesh.collider.clone())
        .unwrap()
        .0
        .clone()
      {
        commands.entity(entity).insert(collider);
      }
    }
  }

  // remove earlier unloaded generations
  generations.next = generations
    .next
    .iter()
    .copied()
    .filter(|gen| gen.0 > latest_loaded.0)
    .collect();
}

fn clean_generation(
  mut commands: Commands,
  mut generations: ResMut<TerrainGenerations>,
  q: Query<(Entity, &TerrainPiece)>,
) {
  // remove generations that have been surpassed
  if generations.next.len() >= 10 {
    info!(
      "pruning surpassed queued generation: {:?}",
      generations.next.first().unwrap()
    );
    generations.next.remove(0);
  }

  // remove the entities of old generations
  for (entity, piece) in q.iter() {
    if piece.generation < generations.current.0 {
      commands.entity(entity).despawn_recursive();
    }
  }
}
