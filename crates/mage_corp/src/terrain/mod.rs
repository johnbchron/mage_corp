mod mesh;
mod region;
use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  ops::Rem,
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
  utils::despawn::DespawnTag,
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
  /// Controls how far from the previous target position the target needs to
  /// move to trigger a terrain regeneration. The trigger distance is equal
  /// to `render_dist / 2.0_f32.powf(render_cube_subdiv_trigger)`.
  /// Should be greater than or equal to
  /// `render_cube_translation_subdiv_increment`, and less than or equal to
  /// `n_sizes`.
  pub render_cube_subdiv_trigger: f32,
  /// Controls the increment in which the render cube will move from the origin
  /// to follow the player. The increment is equal to `render_dist /
  /// 2.0_f32.powf(render_cube_translation_subdiv_increment)`.
  pub render_cube_translation_subdiv_increment: f32,
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

impl TerrainConfig {
  pub fn render_cube_translation_increment(&self) -> f32 {
    self.render_dist
      / 2.0_f32.powf(self.render_cube_translation_subdiv_increment)
  }
  pub fn trigger_distance(&self) -> f32 {
    self.render_dist / 2.0_f32.powf(self.render_cube_subdiv_trigger)
  }
}

impl Default for TerrainConfig {
  fn default() -> Self {
    Self {
      render_dist: 1000.0,
      render_cube_subdiv_trigger: 4.0,
      render_cube_translation_subdiv_increment: 3.0,
      mesh_subdivs: 6,
      mesh_bleed: 1.05,
      n_same_size_meshes: 1,
      n_sizes: 5,
      debug_transform_cubes: false,
    }
  }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TerrainCurrentComposition(pub Composition);

impl TerrainCurrentComposition {
  fn comp_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.0.hash(&mut hasher);
    hasher.finish()
  }
}

impl Default for TerrainCurrentComposition {
  fn default() -> Self {
    let comp = Composition::new(vec![Shape::new_rhai(
      "(sqrt(square(x) + square(y + 5000) + square(z)) - 5000) + ((sin(x / \
       20.0) + sin(y / 20.0) + sin(z / 20.0)) * 4.0)",
    )]);
    Self(comp)
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

impl TerrainCurrentGeneration {
  fn comp_hash(&self) -> u64 {
    let mut hasher = DefaultHasher::new();
    self.comp.hash(&mut hasher);
    hasher.finish()
  }
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
pub struct TerrainTriggerRegeneration {
  pub target_location: Vec3,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Reflect)]
pub enum TerrainEnabledState {
  #[default]
  Enabled,
  Disabled,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainSystemSet;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_state::<TerrainEnabledState>()
      // configure `TerrainMesh`
      .add_asset::<TerrainMesh>()
      .register_type::<TerrainMesh>()
      .register_asset_reflect::<TerrainMesh>()
      // configure `TerrainConfig`
      .register_type::<TerrainConfig>()
      // register other components & resources
      .register_type::<TerrainEnabledState>()
      .register_type::<TerrainDetailTarget>()
      .register_type::<TerrainCurrentGeneration>()
      .register_type::<TerrainNextGeneration>()
      .register_type::<TerrainCurrentComposition>()
      // configure events
      .add_event::<TerrainTriggerRegeneration>()
      // configure set
      .configure_set(
        Update,
        TerrainSystemSet.run_if(in_state(TerrainEnabledState::Enabled)),
      )
      // add systems
      .add_systems(
        Update,
        (
          kickstart_terrain.run_if(
            in_state(TerrainEnabledState::Enabled)
              .and_then(eligible_for_new_gen),
          ),
          init_next_generation.run_if(
            in_state(TerrainEnabledState::Enabled)
              .and_then(on_event::<TerrainTriggerRegeneration>()),
          ),
          flush_assets_from_next_generation.run_if(
            in_state(TerrainEnabledState::Enabled)
              .and_then(resource_exists::<TerrainNextGeneration>()),
          ),
          transition_generations.run_if(
            in_state(TerrainEnabledState::Enabled).and_then(next_gen_is_ready),
          ),
        )
          .chain()
          .in_set(TerrainSystemSet),
      )
      .add_systems(OnEnter(TerrainEnabledState::Enabled), setup_terrain)
      .add_systems(OnExit(TerrainEnabledState::Enabled), cleanup_terrain);
  }
}

fn setup_terrain(mut commands: Commands) {
  commands.insert_resource(TerrainConfig::default());
  commands.insert_resource(TerrainCurrentComposition::default());
  info!("setup terrain resources");
}

fn cleanup_terrain(world: &mut World) {
  // despawn all the current entities
  if let Some(current_gen) = world.remove_resource::<TerrainCurrentGeneration>()
  {
    current_gen.terrain_entities.iter().for_each(|entity| {
      world.entity_mut(*entity).despawn_recursive();
    });
  }

  // cancel all the next generation tasks
  if let Some(next_gen) = world.remove_resource::<TerrainNextGeneration>() {
    // the cancel method consumes the task
    next_gen.mesh_gen_tasks.into_iter().for_each(|task| {
      block_on(task.cancel());
    });
  }

  // remove the rest of the resources
  world.remove_resource::<TerrainConfig>();
  world.remove_resource::<TerrainCurrentComposition>();
}

/// Builds `TerrainNextGeneration` when a `TerrainTriggerRegeneration` event is
/// received.
fn init_next_generation(
  mut commands: Commands,
  mut events: EventReader<TerrainTriggerRegeneration>,
  config: Res<TerrainConfig>,
  current_comp: Res<TerrainCurrentComposition>,
  terrain_meshes: Res<Assets<TerrainMesh>>,
) {
  if let Some(event) = events.iter().next() {
    // hash the current composition
    let comp_hash = current_comp.comp_hash();

    // start a vector for `TerrainMesh`'s which have already been built
    let mut existing_terrain_meshes: Vec<Handle<TerrainMesh>> = vec![];

    // calculate the regions that need to be built
    let regions = region::calculate_regions(&config, event.target_location)
      .iter()
      .filter(|r| {
        // Iterate through all of the terrain meshes to look for ones which
        // match the region and composition hash we're working with.
        // It's fine to iterate through all of them every time because
        // terrain meshes are actually only 88 bytes.
        for (t_mesh_handle_id, t_mesh) in terrain_meshes.iter() {
          if t_mesh.region == **r && t_mesh.comp_hash == comp_hash {
            // build a strong handle out of the handle ID
            let mut handle = Handle::weak(t_mesh_handle_id);
            handle.make_strong(&terrain_meshes);
            existing_terrain_meshes.push(handle);
            debug!("recycling terrain mesh for region: {:?}", r);
            return false;
          }
        }
        true
      })
      .copied()
      .collect::<Vec<TerrainRegion>>();

    // spawn tasks for generating meshes for the regions
    let thread_pool = AsyncComputeTaskPool::get();
    let mesh_gen_tasks = regions
      .clone()
      .into_iter()
      .map(|region| {
        thread_pool.spawn({
          let current_comp = current_comp.0.clone();
          async move { (mesh::generate(&current_comp, &region), region) }
        })
      })
      .collect();

    info!(
      "starting terrain generation with {:?}% recycled regions",
      (existing_terrain_meshes.len() as f32)
        / ((existing_terrain_meshes.len() + regions.len()) as f32)
        * 100.0
    );

    // insert the `TerrainNextGeneration` resource
    commands.insert_resource(TerrainNextGeneration {
      target_location: event.target_location,
      regions,
      mesh_gen_tasks,
      resulting_terrain_meshes: existing_terrain_meshes,
      comp: current_comp.0.clone(),
      comp_hash,
    });
  }
}

fn eligible_for_new_gen(
  current_comp: Res<TerrainCurrentComposition>,
  current_gen: Option<Res<TerrainCurrentGeneration>>,
  next_gen: Option<Res<TerrainNextGeneration>>,
  target_query: Query<&Transform, With<TerrainDetailTarget>>,
  config: Res<TerrainConfig>,
) -> bool {
  // if the next gen is already queued, don't bother
  if next_gen.is_some() {
    return false;
  }
  // if there is neither a current gen nor a next gen, go for it
  if current_gen.is_none() {
    return true;
  }

  // this means we have a current gen but no next gen
  let current_gen = current_gen.unwrap();

  // check if the current composition's hash matches the current gen's comp hash
  if current_comp.comp_hash() != current_gen.comp_hash() {
    return true;
  }

  // check if the target is too far away from the current gen's target
  if let Some(target_transform) = target_query.iter().next() {
    let target_location = target_transform.translation;
    // we compare to 1.0 to give it a little margin
    return (current_gen.target_location
      - current_gen.target_location.rem(config.trigger_distance()))
      != (target_location - target_location.rem(config.trigger_distance()));
  } else {
    error!("no `TerrainDetailTarget` found");
  }
  false
}

/// Sends a `TerrainTriggerRegeneration` event.
fn kickstart_terrain(
  target_query: Query<&Transform, With<TerrainDetailTarget>>,
  mut trigger_regen_events: EventWriter<TerrainTriggerRegeneration>,
) {
  if let Some(target_transform) = target_query.iter().next() {
    info!(
      "sending TerrainTriggerRegeneration event with target: {:?}",
      target_transform.translation
    );
    trigger_regen_events.send(TerrainTriggerRegeneration {
      target_location: target_transform.translation,
    });
  } else {
    error!("no `TerrainDetailTarget` found");
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
      } // no else clause bc a `None` just means the task isn't complete yet
    });

  // we reverse the finished tasks so that the index order from the enumerate
  // is preserved when we `.remove` from mesh_gen_tasks
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
  current_gen: Option<Res<TerrainCurrentGeneration>>,
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
            Name::new(format!("terrain_mesh_{:?}", terrain_mesh_handle.id())),
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

  // despawn the old entities if current_gen exists
  if let Some(current_gen) = current_gen {
    for entity in current_gen.terrain_entities.iter() {
      commands.entity(*entity).insert(DespawnTag);
    }
  } // no else clause because the old current_gen is allowed to not exist

  info!(
    "completed terrain generation with {} terrain entities",
    entites.len()
  );

  commands.insert_resource(TerrainCurrentGeneration {
    target_location:  next_gen.target_location,
    terrain_entities: entites,
    comp:             next_gen.comp.clone(),
  });

  commands.remove_resource::<TerrainNextGeneration>();
}
