use std::time::Instant;

use bevy::{prelude::*, utils::HashMap};

use super::TerrainGenerations;

#[derive(Resource, Reflect, Default)]
struct TerrainGenerationTimings(HashMap<u32, (Instant, Option<Instant>)>);

fn tick_terrain_generation_timings(
  mut timings: ResMut<TerrainGenerationTimings>,
  generations: Res<TerrainGenerations>,
) {
  // all generation indices in the current and next generations
  let existent_generations = generations
    .next
    .iter()
    .map(|(i, _)| *i)
    .chain(Some(generations.current.0))
    .collect::<Vec<_>>();

  // make sure they're all in the timings
  for generation in existent_generations {
    if !timings.0.contains_key(&generation) {
      timings.0.insert(generation, (Instant::now(), None));
    }
  }

  // complete the current generation if it's not complete (and log it)
  if let Some((start, None)) = timings.0.get(&generations.current.0) {
    if generations.current.0 == 0 {
      return;
    }

    *timings.0.get_mut(&generations.current.0).unwrap() =
      (*start, Some(Instant::now()));
    info!(
      "generation {} complete in {:?}",
      generations.current.0,
      timings.0.get(&generations.current.0).unwrap().1.unwrap()
        - timings.0.get(&generations.current.0).unwrap().0
    );
  }
}

pub struct TerrainGenerationTimingPlugin;

impl Plugin for TerrainGenerationTimingPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<TerrainGenerationTimings>()
      .add_systems(Update, tick_terrain_generation_timings);
  }
}
