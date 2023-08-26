use futures_lite::future::FutureExt;
use crate::terrain::mesh::TerrainMesh;
use crate::terrain::region::TerrainRegion;
use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher}, task::Poll,
};
mod mesh;
mod region;

use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use planiscope::{builder::box_, comp::Composition};

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
  /// Controls the minimum number of same-sized meshes that form the border
  /// between two sizes.
  pub n_same_size_meshes: u8,
  /// Controls how many times to subdivide the world box.
  pub n_sizes: u8,
}

impl Default for TerrainConfig {
  fn default() -> Self {
    Self {
      render_dist: 500.0,
      render_cube_translation_increment: 50.0,
      mesh_max_subdivs: 6,
      mesh_min_subdivs: 3,
      n_same_size_meshes: 2,
      n_sizes: 3,
    }
  }
}

#[derive(Resource)]
pub struct TerrainCurrentComposition {
  pub comp: Composition,
}

impl Default for TerrainCurrentComposition {
  fn default() -> Self {
    let mut comp = Composition::new();
    comp.add_shape(box_(100.0, 1.0, 100.0), [0.0, -0.5, 0.0]);
    Self { comp }
  }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainCurrentGeneration {
  pub target_location:  Vec3,
  pub terrain_entities: Vec<Entity>,
  #[reflect(ignore)]
  pub comp:             Composition,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainNextGeneration {
  pub target_location: Vec3,
  pub regions:         Vec<region::TerrainRegion>,
  #[reflect(ignore)]
  pub mesh_gen_tasks:   Vec<Task<(Mesh, TerrainRegion)>>,
  pub resulting_terrain_meshes: Vec<Handle<TerrainMesh>>,
  #[reflect(ignore)]
  pub comp:            Composition,
  pub comp_hash:       u64,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<TerrainDetailTarget>()
      .register_type::<TerrainConfig>()
      .register_type::<TerrainCurrentGeneration>()
      .register_type::<TerrainNextGeneration>()
      .init_resource::<TerrainConfig>()
      .init_resource::<TerrainCurrentComposition>()
      .add_systems(Update, kickstart_terrain_if_none);
  }
}

fn init_next_generation(
  commands: &mut Commands,
  target_location: Vec3,
  config: TerrainConfig,
  current_comp: Composition,
) {
  let regions =
    region::calculate_regions(&config, target_location);
  let thread_pool = AsyncComputeTaskPool::get();

  let mut hasher = DefaultHasher::new();
  current_comp.hash(&mut hasher);
  let comp_hash = hasher.finish();

  commands.insert_resource(TerrainNextGeneration {
    target_location: target_location,
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
      info!("running `kickstart_terrain_if_none`");

      init_next_generation(
        &mut commands,
        target_transform.translation,
        config.clone(),
        current_comp.comp.clone(),
      )
    }
  }
}

fn flush_assets_from_next_generation(
  mut commands: Commands,
  next_gen: Option<ResMut<TerrainNextGeneration>>,
  meshes: ResMut<Assets<Mesh>>,
  terrain_meshes: ResMut<Assets<TerrainMesh>>,
) {
  if let Some(next_gen) = next_gen {
    next_gen.mesh_gen_tasks.iter().enumerate().for_each(|(index, task)| {
      match task.poll() {
        Poll::Ready((mesh, region)) => {
          terrain_meshes.add(
            TerrainMesh {
              mesh: meshes.add(mesh),
              region,
              comp_hash: next_gen.comp_hash,
            }
          );
        },
        Poll::Pending => (),
      }
    })
  }
}