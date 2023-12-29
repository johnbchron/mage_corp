use tracing::info_span;

use crate::{
  bufmesh::{BufMesh, FullVertex},
  mizu::MizuMesh,
};

/// Simplifies a mesh by merging coplanar faces.
pub fn simplify_mesh(mesh: BufMesh) -> BufMesh {
  let _span = info_span!("mosh::simplify_mesh::from_buffer").entered();

  let vertices = mesh
    .positions
    .iter()
    .zip(mesh.normals.iter())
    .map(|(p, n)| FullVertex {
      position: *p,
      normal:   *n,
    })
    .collect::<Vec<_>>();
  let mut mizu = MizuMesh::from_buffers(&vertices, &mesh.triangles);

  drop(_span);
  let _span = info_span!("mosh::simplify_mesh::simplify").entered();
  // simplification goes here

  drop(_span);
  let _span = info_span!("mosh::simplify_mesh::to_buffers").entered();

  let (vertices, faces) = mizu.to_buffers();
  let mesh = BufMesh {
    positions: vertices.iter().map(|v| v.position).collect::<Vec<_>>(),
    normals:   vertices.iter().map(|v| v.normal).collect::<Vec<_>>(),
    triangles: faces,
  };
  mesh
}
