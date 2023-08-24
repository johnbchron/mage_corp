mod mesh;

use bevy::prelude::*;

#[derive(Component)]
pub struct TerrainDetailTarget;

#[derive(Resource)]
pub struct TerrainCurrentGeneration {
  pub target_location:  Vec3,
  pub terrain_entities: Vec<Entity>,
}
