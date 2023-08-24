use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use bevy::{
  prelude::*,
  reflect::{TypePath, TypeUuid},
};
use planiscope::{
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
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

impl TerrainMesh {
  /// Generate a `TerrainMesh` from a `Composition` and a `TerrainRegion`.
  pub fn generate(comp: &Composition, region: &TerrainRegion) -> TerrainMesh {
    // start a new fidget context
    let mut ctx = fidget::Context::new();

    // maybe make this configurable?
    let compilation_settings = CompilationSettings {
      min_voxel_size: 0.01,
    };

    // get the node for the solid field
    let solid_root_node = comp.compile_solid(&mut ctx, &compilation_settings);
    // transform the desired region into -1..1
    let solid_root_node = planiscope::nso::nso_normalize_region(
      solid_root_node,
      [region.position.x, region.position.y, region.position.z],
      [region.scale.x, region.scale.y, region.scale.z],
      &mut ctx,
    );

    // get the tape
    let solid_tape: fidget::eval::Tape<fidget::vm::Eval> =
      ctx.get_tape(solid_root_node).unwrap();

    // tesselate
    let mut full_mesh = FullMesh::tesselate(
      &solid_tape,
      None,
      true,
      region.max_subdivs,
      region.max_subdivs,
    );

    // remove any vertices outside -1..1
    full_mesh.prune();
    // might not want to translate, not sure
    full_mesh.transform(region.position.into(), region.scale.into());

    // compute the hash of the composition
    let mut hasher = DefaultHasher::new();
    comp.hash(&mut hasher);
    let comp_hash = hasher.finish();

    TerrainMesh {
      mesh: full_mesh.into(),
      region: region.clone(),
      comp_hash,
    }
  }
}
