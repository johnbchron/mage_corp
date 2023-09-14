mod cache;

use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use planiscope::{
  comp::Composition,
  mesher::{FastSurfaceNetsMesher, FullMesh, Mesher, MesherInputs},
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
  let mesher_inputs = MesherInputs {
    position: region.position.into(),
    scale:    region.scale.into(),
    subdivs:  region.subdivs,
    prune:    true,
  };

  let meta_hash = cache::mesh_meta_hash(comp, region);

  match cache::read_mesh_from_file(meta_hash) {
    Some(mesh) => {
      debug!("read mesh from file");
      bevy_mesh_from_pls_mesh(mesh)
    }
    None => {
      let full_mesh =
        FastSurfaceNetsMesher::build_mesh(comp, mesher_inputs).unwrap();
      if let Some(path) = cache::write_mesh_to_file(meta_hash, &full_mesh) {
        debug!("wrote mesh to {}", path);
      }
      let mesh: Mesh = bevy_mesh_from_pls_mesh(full_mesh);
      if mesh.count_vertices() != 0 {
        debug!(
          "generated terrain mesh for position {:?} and scale {:?} with {:?} \
           vertices",
          region.position,
          region.scale,
          mesh.count_vertices()
        );
      }
      mesh
    }
  }
}

fn bevy_mesh_from_pls_mesh(mesh: FullMesh) -> Mesh {
  let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);

  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_POSITION,
    mesh
      .vertices
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );
  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_NORMAL,
    mesh
      .normals
      .into_iter()
      .map(|v| v.to_array())
      .collect::<Vec<_>>(),
  );

  bevy_mesh.set_indices(Some(Indices::U32(
    mesh
      .triangles
      .into_iter()
      .flat_map(|v| v.to_array())
      .collect(),
  )));
  bevy_mesh
}
