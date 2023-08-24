mod gen;

use bevy::{
  prelude::*,
  reflect::{TypePath, TypeUuid},
};

#[derive(Debug, Clone)]
pub struct TerrainRegion {
  pub position:    Vec3,
  pub scale:       Vec3,
  pub max_subdivs: u8,
  pub min_subdivs: u8,
}

#[derive(Debug, TypeUuid, TypePath)]
#[uuid = "3dc0b7c0-e829-4634-b490-2f5f53873a1d"]
pub struct TerrainMesh {
  /// Contains the bevy mesh for this terrain mesh.
  mesh:      Mesh,
  /// Describes the region that the composition was evaluated over.
  region:    TerrainRegion,
  /// The hash of the composition.
  comp_hash: u64,
}
