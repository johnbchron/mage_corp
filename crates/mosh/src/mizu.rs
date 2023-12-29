//! # Mizu
//! Mizu is an intelligent mesh data structure built for mesh simplification.

mod vertex;

use std::sync::OnceLock;

use rayon::prelude::*;

pub use self::vertex::{Vertex, VertexData};

/// A face in a [`MizuMesh`].
pub struct Face {
  vertices: glam::UVec3,
  normal:   glam::Vec3A,
}

/// A mesh data structure for mesh simplification.
pub struct MizuMesh<D: VertexData> {
  vertices:  Vec<Vertex<D>>,
  faces:     Vec<Face>,
  opposites: OnceLock<Vec<[Option<u32>; 3]>>,
}

impl<D: VertexData> MizuMesh<D> {
  /// Creates a new `MizuMesh` from the given vertices and faces.
  pub fn from_buffers(vertices: &[D], faces: &[glam::UVec3]) -> Self {
    let mut mesh = Self {
      vertices:  vertices.iter().map(|v| Vertex::new(v.clone())).collect(),
      faces:     Vec::with_capacity(faces.len()),
      opposites: OnceLock::new(),
    };
    let faces = faces
      .par_iter()
      .map(|f| {
        let normal = mesh.compute_normal(f);
        Face {
          vertices: *f,
          normal,
        }
      })
      .collect::<Vec<_>>();
    mesh.faces = faces;
    mesh
  }

  /// Returns the number of vertices in the mesh.
  pub fn vertex_count(&self) -> u32 { self.vertices.len() as u32 }
  /// Returns a reference to the vertex at the given index.
  pub fn vertex(&self, index: u32) -> &Vertex<D> {
    &self.vertices[index as usize]
  }

  /// Computes the normal of the face with the given indices.
  pub fn compute_normal(&self, indices: &glam::UVec3) -> glam::Vec3A {
    let a = self.vertex(indices.x).pos();
    let b = self.vertex(indices.y).pos();
    let c = self.vertex(indices.z).pos();
    (b - a).cross(c - a).normalize()
  }
}
