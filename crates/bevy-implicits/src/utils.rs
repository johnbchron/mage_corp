use bevy::prelude::*;
use planiscope::mesher::BufMesh;

/// Converts a `planiscope::mesher::FullMesh` to a `bevy::render::mesh::Mesh`.
pub fn bevy_mesh_from_pls_mesh(mesh: BufMesh) -> Mesh {
  let mut bevy_mesh =
    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

  bevy_mesh.insert_attribute(
    Mesh::ATTRIBUTE_POSITION,
    mesh
      .positions
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

  bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
    mesh
      .triangles
      .into_iter()
      .flat_map(|v| v.to_array())
      .collect(),
  )));
  bevy_mesh
}
