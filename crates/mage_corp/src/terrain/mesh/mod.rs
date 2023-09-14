mod cache;

use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_xpbd_3d::prelude::*;
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
  /// The collider for the generated mesh
  #[reflect(ignore)]
  pub collider:  Option<Collider>,
  /// The hash of the composition.
  pub comp_hash: u64,
}

pub fn build_mesh_and_collider(
  comp: &Composition,
  region: &TerrainRegion,
) -> (Mesh, Option<Collider>) {
  let (full_mesh, collider) = generate_or_fetch_pack(comp, region);

  (bevy_mesh_from_pls_mesh(full_mesh), collider)
}

pub fn generate_collider(full_mesh: FullMesh) -> Option<Collider> {
  if full_mesh.triangles.len() == 0 {
    return None;
  }
  Some(Collider::convex_decomposition(
    full_mesh.vertices.into_iter().map(|v| v.into()).collect(),
    full_mesh
      .triangles
      .into_iter()
      .map(|v| v.to_array())
      .collect(),
  ))
}

pub fn generate_full_mesh(
  comp: &Composition,
  region: &TerrainRegion,
) -> FullMesh {
  let mesher_inputs = MesherInputs {
    position: region.position.into(),
    scale:    region.scale.into(),
    subdivs:  region.subdivs,
    prune:    true,
  };

  FastSurfaceNetsMesher::build_mesh(comp, mesher_inputs).unwrap()
}

pub fn generate_or_fetch_pack(
  comp: &Composition,
  region: &TerrainRegion,
) -> (FullMesh, Option<Collider>) {
  let meta_hash = cache::mesh_meta_hash(comp, region);

  let optional_pack: Option<(FullMesh, Option<Collider>)> =
    cache::read_pack_from_file(meta_hash).map(|o| o.into());
  match optional_pack {
    Some(pack) => {
      debug!("read pack from file");
      pack
    }
    None => {
      let full_mesh = generate_full_mesh(comp, region);
      let collider = generate_collider(full_mesh.clone());
      let pack = (full_mesh, collider);

      if pack.0.vertices.len() != 0 {
        debug!(
          "generated terrain mesh for position {:?} and scale {:?} with {:?} \
           vertices",
          region.position,
          region.scale,
          pack.0.vertices.len()
        );
      }
      if let Some(path) =
        cache::write_pack_to_file(meta_hash, &pack.clone().into())
      {
        debug!("wrote pack to {}", path);
      }
      pack
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
