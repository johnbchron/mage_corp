use meshopt2::{
  packing::DecodePosition,
  simplify::{simplify_decoder},
  simplify::SimplifyOptions
};

use crate::bufmesh::BufMesh;

struct Position(glam::Vec3A);

impl DecodePosition for Position {
  fn decode_position(&self) -> [f32; 3] { self.0.to_array() }
}

/// Simplifies a mesh by merging coplanar faces.
pub fn simplify_mesh(mesh: BufMesh) -> BufMesh {
  let original_triangle_count = mesh.triangles.len();
  let vertices = mesh.positions.into_iter().map(Position).collect::<Vec<_>>();
  let indices = mesh
    .triangles
    .into_iter()
    .flat_map(|i| [i.x as u32, i.y as u32, i.z as u32])
    .collect::<Vec<_>>();
  let normals = mesh.normals;
  // .into_iter()
  // .map(|v| v.to_array())
  // .collect::<Vec<_>>();

  let target_count = (original_triangle_count as f32 * 0.2) as usize;
  let new_indices = simplify_decoder(
    &indices,
    &vertices,
    target_count,
    0.001,
    SimplifyOptions::None,
    None,
  );

  let triangles = new_indices
    .into_iter()
    .map_windows(|i: &[u32; 3]| glam::UVec3::from_array(*i))
    .collect::<Vec<_>>();
  let positions = vertices.into_iter().map(|v| v.0).collect::<Vec<_>>();
  BufMesh {
    normals,
    positions,
    triangles,
  }
}
