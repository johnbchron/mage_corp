use std::sync::OnceLock;

use rayon::prelude::*;
use tracing::info_span;

use super::{MizuMesh, VertexData};
use crate::mizu::{Face, Vertex};

impl<D: VertexData> MizuMesh<D> {
  /// Creates a new `MizuMesh` from the given vertices and faces.
  pub fn from_buffers(vertices: &[D], faces: &[glam::UVec3]) -> Self {
    let _span = info_span!("mosh::MizuMesh::from_buffers").entered();

    let mut mesh = Self {
      vertices:  vertices.iter().map(|v| Vertex::new(v.clone())).collect(),
      faces:     Vec::with_capacity(faces.len()),
      opposites: OnceLock::new(),
    };
    let faces = faces
      .par_iter()
      .map(|f| {
        let normal = mesh.compute_normal(f);
        Face::new(*f, normal)
      })
      .collect::<Vec<_>>();
    mesh.faces = faces;
    mesh.opposites.set(mesh.build_opposites()).unwrap();

    mesh
  }
  /// Builds buffers from the mesh.
  pub fn to_buffers(&self) -> (Vec<D>, Vec<glam::UVec3>) {
    let _span = info_span!("mosh::MizuMesh::to_buffers").entered();

    let vertices = self.vertices.iter().map(|v| v.data().clone()).collect();
    let faces = self.faces.iter().map(|f| *f.vertices()).collect();
    (vertices, faces)
  }
}
