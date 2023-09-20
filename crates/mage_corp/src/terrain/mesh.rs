use bevy::{
  prelude::*,
  reflect::TypeUuid,
  render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_xpbd_3d::prelude::*;
use planiscope::mesher::FullMesh;

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

pub fn bevy_mesh_from_pls_mesh(mesh: FullMesh) -> Mesh {
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
