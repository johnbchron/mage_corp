use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use planiscope::{
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
};

use super::*;

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
