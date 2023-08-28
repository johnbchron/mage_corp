use bevy::{prelude::*, reflect::TypeUuid};
use planiscope::{
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
};

use super::region::TerrainRegion;

#[derive(Debug, TypeUuid, Reflect)]
#[uuid = "3dc0b7c0-e829-4634-b490-2f5f53873a1d"]
pub struct TerrainMesh {
  /// Contains the bevy mesh for this terrain mesh.
  pub mesh:      Handle<Mesh>,
  /// Describes the region that the composition was evaluated over.
  pub region:    TerrainRegion,
  /// The hash of the composition.
  pub comp_hash: u64,
}

pub fn generate(comp: &Composition, region: &TerrainRegion) -> Mesh {
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
    region.min_subdivs,
  );

  // remove any vertices outside -1..1
  full_mesh.prune();
  // scale but don't translate; we'll do that with bevy
  full_mesh.transform(Vec3::ZERO.into(), region.scale.into());

  let mesh: Mesh = full_mesh.into();
  if mesh.count_vertices() != 0 {
    info!(
      "generated terrain mesh for position {:?} and scale {:?} with {:?} \
       vertices",
      region.position,
      region.scale,
      mesh.count_vertices()
    );
  }

  mesh
}
