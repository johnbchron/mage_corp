use bevy::{prelude::*, reflect::TypeUuid};
use bevy_xpbd_3d::prelude::*;

use super::region::TerrainRegion;

#[derive(Debug, TypeUuid, Asset, Reflect)]
#[uuid = "3dc0b7c0-e829-4634-b490-2f5f53873a1d"]
pub struct TerrainMesh {
  /// Contains the bevy mesh for this terrain mesh.
  pub mesh:      Handle<Mesh>,
  /// Describes the region that the composition was evaluated over.
  pub region:    TerrainRegion,
  /// The collider for the generated mesh
  #[reflect(ignore)]
  pub collider:  Option<Collider>,
  /// The hash of the composition.
  pub comp_hash: u64,
}
