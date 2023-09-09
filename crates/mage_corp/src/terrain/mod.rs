mod components;
mod mesh;
mod region;

use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future::{block_on, poll_once};
use planiscope::{
  builder::{box_, sphere},
  comp::Composition,
};

use crate::{
  materials::toon::ToonMaterial,
  terrain::{mesh::TerrainMesh, region::TerrainRegion},
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TerrainDetailTarget;

#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct TerrainConfig {
  /// The half-size of the render cube to build (smallest distance from player
  /// to edge).
  pub render_dist: f32,
  /// Controls the increment in which the render cube will move from the origin
  /// to follow the player. Should change proportionally to render_dist.
  pub render_cube_translation_increment: f32,
  /// Controls the maximum subdivisions of each mesh.
  pub mesh_max_subdivs: u8,
  /// Controls the minimum number of subdivisions of each mesh.
  pub mesh_min_subdivs: u8,
  /// Controls how much each mesh bleeds into the next.
  pub mesh_bleed: f32,
  /// Controls the minimum number of same-sized meshes that form the border
  /// between two sizes.
  pub n_same_size_meshes: u8,
  /// Controls how many times to subdivide the world box.
  pub n_sizes: u8,
  /// Whether to place 1/8th scale cubes at the position of each mesh.
  pub debug_transform_cubes: bool,
}

impl Default for TerrainConfig {
  fn default() -> Self {
    Self {
      render_dist: 500.0,
      render_cube_translation_increment: 500.0 / 8.0,
      mesh_max_subdivs: 7,
      mesh_min_subdivs: 1,
      mesh_bleed: 1.1,
      n_same_size_meshes: 1,
      n_sizes: 5,
      debug_transform_cubes: false,
    }
  }
}

#[derive(Resource)]
pub struct TerrainCurrentComposition {
  pub comp: Composition<components::HillyLand>,
}

impl Default for TerrainCurrentComposition {
  fn default() -> Self {
    let mut comp = Composition::new();
    comp.add_shape(components::HillyLand {}, [0.0, 0.0, 0.0]);
    Self { comp }
  }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainCurrentGeneration {
  pub target_location:  Vec3,
  pub terrain_entities: Vec<Entity>,
  #[reflect(ignore)]
  pub comp:             Composition<components::HillyLand>,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainNextGeneration {
  pub target_location:          Vec3,
  pub regions:                  Vec<region::TerrainRegion>,
  #[reflect(ignore)]
  pub mesh_gen_tasks:           Vec<Task<(Mesh, TerrainRegion)>>,
  pub resulting_terrain_meshes: Vec<Handle<TerrainMesh>>,
  #[reflect(ignore)]
  pub comp:                     Composition<components::HillyLand>,
  pub comp_hash:                u64,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_asset::<TerrainMesh>()
      .register_type::<TerrainMesh>()
      .register_asset_reflect::<TerrainMesh>()
      .register_type::<TerrainDetailTarget>()
      .register_type::<TerrainConfig>()
      .register_type::<TerrainCurrentGeneration>()
      .register_type::<TerrainNextGeneration>()
      .init_resource::<TerrainConfig>()
      .init_resource::<TerrainCurrentComposition>()
      .add_systems(Update, kickstart_terrain_if_none)
      .add_systems(Update, flush_assets_from_next_generation)
      .add_systems(Update, spawn_next_generation_entities);
  }
}

fn init_next_generation(
  commands: &mut Commands,
  target_location: Vec3,
  config: TerrainConfig,
  current_comp: Composition<components::HillyLand>,
) {
  let regions = region::calculate_regions_with_static_render_cube_origin(
    &config,
    target_location,
  );
  let thread_pool = AsyncComputeTaskPool::get();

  let mut hasher = DefaultHasher::new();
  current_comp.hash(&mut hasher);
  let comp_hash = hasher.finish();

  commands.insert_resource(TerrainNextGeneration {
    target_location,
    regions: regions.clone(),
    mesh_gen_tasks: regions
      .into_iter()
      .map(|region| {
        thread_pool.spawn({
          let current_comp = current_comp.clone();
          async move { (mesh::generate(&current_comp, &region), region) }
        })
      })
      .collect(),
    resulting_terrain_meshes: vec![],
    comp: current_comp,
    comp_hash,
  });
}

/// Builds `TerrainNextGeneration` if neither `TerrainCurrentGeneration` nor
/// `TerrainNextGeneration` exist.
fn kickstart_terrain_if_none(
  mut commands: Commands,
  target_query: Query<&Transform, With<TerrainDetailTarget>>,
  current_gen: Option<Res<TerrainCurrentGeneration>>,
  next_gen: Option<Res<TerrainNextGeneration>>,
  config: Res<TerrainConfig>,
  current_comp: Res<TerrainCurrentComposition>,
) {
  if let Some(target_transform) = target_query.iter().next() {
    if current_gen.is_none() && next_gen.is_none() {
      // info!("running `kickstart_terrain_if_none`");

      init_next_generation(
        &mut commands,
        target_transform.translation,
        config.clone(),
        current_comp.comp.clone(),
      )
    }
  }
}

/// Moves built meshes out of tasks, adds them as assets, and stores them in
/// `TerrainNextGeneration`.
fn flush_assets_from_next_generation(
  next_gen: Option<ResMut<TerrainNextGeneration>>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut terrain_meshes: ResMut<Assets<TerrainMesh>>,
) {
  if let Some(mut next_gen) = next_gen {
    let mut finished_tasks = vec![];
    let comp_hash = next_gen.comp_hash;
    next_gen
      .mesh_gen_tasks
      .iter_mut()
      .enumerate()
      .for_each(|(index, task)| {
        if let Some((mesh, region)) = block_on(poll_once(task)) {
          finished_tasks.push((
            index,
            terrain_meshes.add(TerrainMesh {
              mesh: meshes.add(mesh),
              region,
              comp_hash,
            }),
          ));
        }
      });

    finished_tasks.reverse();
    for (index, terrain_mesh) in finished_tasks {
      std::mem::drop(next_gen.mesh_gen_tasks.remove(index));
      next_gen.resulting_terrain_meshes.push(terrain_mesh);
      // info!("flushed terrain mesh {}", index);
    }
  }
}

/// If `TerrainNextGeneration` is fully built, spawns entities from it and
/// creates `TerrainCurrentGeneration`.
fn spawn_next_generation_entities(
  mut commands: Commands,
  next_gen: Option<Res<TerrainNextGeneration>>,
  terrain_meshes: Res<Assets<TerrainMesh>>,
  config: Res<TerrainConfig>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  if let Some(next_gen) = next_gen {
    if !next_gen.mesh_gen_tasks.is_empty() {
      return;
    }

    let entites = next_gen
      .resulting_terrain_meshes
      .iter()
      .map(|terrain_mesh_handle| {
        if let Some(terrain_mesh) = terrain_meshes.get(terrain_mesh_handle) {
          let entity = commands
            .spawn((
              SpatialBundle::from_transform(Transform::from_translation(
                terrain_mesh.region.position,
              )),
              terrain_mesh.mesh.clone(),
              terrain_mesh_handle.clone(),
              toon_materials.add(ToonMaterial {
                color: Color::rgb(0.180, 0.267, 0.169),
                outline_scale: 0.0,
                ..default()
              }),
            ))
            .id();

          if config.debug_transform_cubes {
            commands.entity(entity).with_children(|parent| {
              parent.spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
                material: toon_materials.add(ToonMaterial {
                  color: Color::WHITE,
                  outline_scale: 0.0,
                  ..default()
                }),
                transform: Transform::from_scale(
                  terrain_mesh.region.scale / 8.0,
                ),
                ..default()
              });
            });
          }
          Some(entity)
        } else {
          None
        }
      })
      .filter(|entity| entity.is_some())
      .map(|entity| entity.unwrap())
      .collect::<Vec<Entity>>();

    commands.insert_resource(TerrainCurrentGeneration {
      target_location:  next_gen.target_location,
      terrain_entities: entites,
      comp:             next_gen.comp.clone(),
    });

    commands.remove_resource::<TerrainNextGeneration>();
  }
}
