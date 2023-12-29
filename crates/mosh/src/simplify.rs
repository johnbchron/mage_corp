use crate::{bufmesh::BufMesh, hedge::HedgeMesh};

/// Simplifies a mesh by merging coplanar faces.
pub fn simplify_mesh(mesh: BufMesh) -> BufMesh {
  let mut hedge = HedgeMesh::from_bufmesh(mesh);

  hedge.regenerate_invalid_keys();
  hedge.merge_coplanar_faces(0.2);

  hedge.to_bufmesh()
}
