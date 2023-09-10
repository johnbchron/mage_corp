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
use planiscope::{comp::Composition, shape::Shape};

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
  pub mesh_subdivs: u8,
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
      mesh_subdivs: 6,
      mesh_bleed: 1.05,
      n_same_size_meshes: 1,
      n_sizes: 5,
      debug_transform_cubes: false,
    }
  }
}

#[derive(Resource)]
pub struct TerrainCurrentComposition {
  pub comp: Composition,
}

impl Default for TerrainCurrentComposition {
  fn default() -> Self {
    let comp = Composition::new(vec![Shape::new_rhai(
      "(sqrt(square(x) + square(y + 5000) + square(z)) - 5000) + ((sin(x / \
       20.0) + sin(y / 20.0) + sin(z / 20.0)) * 4.0)",
    )]);
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
  pub target_location:          Vec3,
  pub regions:                  Vec<region::TerrainRegion>,
  #[reflect(ignore)]
  pub mesh_gen_tasks:           Vec<Task<(Mesh, TerrainRegion)>>,
  pub resulting_terrain_meshes: Vec<Handle<TerrainMesh>>,
  #[reflect(ignore)]
  pub comp:                     Composition,
  pub comp_hash:                u64,
}

#[derive(Event)]
struct TerrainTriggerRegeneration {
  pub target_location: Vec3,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      // configure `TerrainMesh`
      .add_asset::<TerrainMesh>()
      .register_type::<TerrainMesh>()
      .register_asset_reflect::<TerrainMesh>()
      // configure `TerrainConfig`
      .register_type::<TerrainConfig>()
      .init_resource::<TerrainConfig>()
      // register other components & resources
      .register_type::<TerrainDetailTarget>()
      .register_type::<TerrainCurrentGeneration>()
      .register_type::<TerrainNextGeneration>()
      .init_resource::<TerrainCurrentComposition>()
      // configure events
      .add_event::<TerrainTriggerRegeneration>()
      // add systems
      .add_systems(
        Update,
        (
          (
            kickstart_terrain.run_if(no_terrain_gens_exist),
            init_next_generation
              .run_if(on_event::<TerrainTriggerRegeneration>()),
          )
            .chain(),
          (
            flush_assets_from_next_generation
              .run_if(resource_exists::<TerrainNextGeneration>()),
            transition_generations.run_if(next_gen_is_ready),
          )
            .chain(),
        ),
      );
  }
}

/// Builds `TerrainNextGeneration` when a `TerrainTriggerRegeneration` event is
/// received.
fn init_next_generation(
  mut commands: Commands,
  mut events: EventReader<TerrainTriggerRegeneration>,
  config: Res<TerrainConfig>,
  current_comp: Res<TerrainCurrentComposition>,
) {
  if let Some(event) = events.iter().next() {
    // calculate the regions that need to be built
    let regions = region::calculate_regions_with_static_render_cube_origin(
      &config,
      event.target_location,
    );

    // hash the current composition
    let mut hasher = DefaultHasher::new();
    current_comp.comp.hash(&mut hasher);
    let comp_hash = hasher.finish();

    // spawn tasks for generating meshes for the regions
    let thread_pool = AsyncComputeTaskPool::get();
    let mesh_gen_tasks = regions
      .clone()
      .into_iter()
      .map(|region| {
        thread_pool.spawn({
          let current_comp = current_comp.comp.clone();
          async move { (mesh::generate(&current_comp, &region), region) }
        })
      })
      .collect();

    // insert the `TerrainNextGeneration` resource
    commands.insert_resource(TerrainNextGeneration {
      target_location: event.target_location,
      regions,
      mesh_gen_tasks,
      resulting_terrain_meshes: vec![],
      comp: current_comp.comp.clone(),
      comp_hash,
    });
  }
}

/// Returns `true` if neither `TerrainCurrentGeneration` nor
/// `TerrainNextGeneration` exist.
fn no_terrain_gens_exist(
  current_gen: Option<Res<TerrainCurrentGeneration>>,
  next_gen: Option<Res<TerrainNextGeneration>>,
) -> bool {
  current_gen.is_none() && next_gen.is_none()
}

/// Sends a `TerrainTriggerRegeneration` event.
fn kickstart_terrain(
  target_query: Query<&Transform, With<TerrainDetailTarget>>,
  mut trigger_regen_events: EventWriter<TerrainTriggerRegeneration>,
) {
  if let Some(target_transform) = target_query.iter().next() {
    trigger_regen_events.send(TerrainTriggerRegeneration {
      target_location: target_transform.translation,
    });
  }
}

/// Runs in service of `TerrainNextGeneration`. Moves task results from the
/// `TerrainNextGeneration` resource and adds them as `Mesh` and `TerrainMesh`
/// assets.
fn flush_assets_from_next_generation(
  mut next_gen: ResMut<TerrainNextGeneration>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut terrain_meshes: ResMut<Assets<TerrainMesh>>,
) {
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
  }
}

/// Indicates whether `TerrainNextGeneration` is fully built.
fn next_gen_is_ready(next_gen: Option<Res<TerrainNextGeneration>>) -> bool {
  if let Some(next_gen) = next_gen {
    return next_gen.mesh_gen_tasks.is_empty();
  }
  false
}

/// If `TerrainNextGeneration` is fully built, spawns entities from it and
/// creates `TerrainCurrentGeneration`.
fn transition_generations(
  mut commands: Commands,
  next_gen: Res<TerrainNextGeneration>,
  terrain_meshes: Res<Assets<TerrainMesh>>,
  config: Res<TerrainConfig>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  let entites = next_gen
    .resulting_terrain_meshes
    .iter()
    .filter_map(|terrain_mesh_handle| {
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
              transform: Transform::from_scale(terrain_mesh.region.scale / 8.0),
              ..default()
            });
          });
        }
        Some(entity)
      } else {
        error!(
          "TerrainMesh not found for terrain mesh handle: {:?}",
          terrain_mesh_handle
        );
        None
      }
    })
    .collect::<Vec<Entity>>();

  commands.insert_resource(TerrainCurrentGeneration {
    target_location:  next_gen.target_location,
    terrain_entities: entites,
    comp:             next_gen.comp.clone(),
  });

  commands.remove_resource::<TerrainNextGeneration>();
}
