use crate::{
  bufmesh::{BufMesh, FullVertex},
  hedge::HedgeMesh,
};

/// Simplifies a mesh by merging coplanar faces.
pub fn simplify_mesh(mesh: BufMesh) -> BufMesh {
  let triangles = mesh
    .triangles
    .iter()
    .map(|t| (t.x as usize, t.y as usize, t.z as usize))
    .collect::<Vec<_>>();
  let vertices = (0..mesh.positions.len())
    .map(|i| FullVertex {
      position: mesh.positions[i],
      normal:   mesh.normals[i],
    })
    .collect::<Vec<_>>();

  let mut hedge =
    HedgeMesh::from_buffers(triangles.as_slice(), vertices.as_slice());

  // simplification goes here
  hedge.regenerate_invalid_keys();
  hedge.prune_unused_vertices();
  hedge.dedup_equal_vertices();
  for face in hedge.faces() {
    hedge.is_valid_face(face).unwrap();
  }
  for group in hedge
    .find_coplanar_face_groups()
    .into_iter()
    .filter(|g| g.len() > 1)
  {
    hedge.merge_face_group(group);
  }

  let hedge_buffers = hedge.to_buffers();
  let triangles = hedge_buffers
    .0
    .into_iter()
    .map(|t| glam::UVec3::new(t.0 as u32, t.1 as u32, t.2 as u32))
    .collect::<Vec<_>>();
  let vertices = hedge_buffers
    .1
    .iter()
    .map(|v| v.position)
    .collect::<Vec<_>>();
  let normals = hedge_buffers
    .1
    .into_iter()
    .map(|v| v.normal)
    .collect::<Vec<_>>();

  BufMesh {
    triangles,
    positions: vertices,
    normals,
  }
}
