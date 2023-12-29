//! # Mizu
//! Mizu is an intelligent mesh data structure built for mesh simplification.

mod buffers;
mod face;
mod vertex;

use std::sync::OnceLock;

use hashbrown::HashMap;
use rayon::prelude::*;
use tracing::info_span;

pub use self::{
  face::Face,
  vertex::{Vertex, VertexData},
};

/// A mesh data structure for mesh simplification.
pub struct MizuMesh<D: VertexData> {
  vertices:  Vec<Vertex<D>>,
  faces:     Vec<Face>,
  opposites: OnceLock<Vec<[Option<u32>; 3]>>,
}

impl<D: VertexData> MizuMesh<D> {
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

  /// Returns a slice of the opposites of each face in the mesh.
  pub fn opposites(&self) -> &[[Option<u32>; 3]] {
    self.opposites.get_or_init(|| self.build_opposites())
  }

  fn build_opposites(&self) -> Vec<[Option<u32>; 3]> {
    let _span = info_span!("mosh::MizuMesh::build_opposites").entered();

    let arc_to_face_map = self
      .faces
      .par_iter()
      .enumerate()
      .flat_map(|(i, face)| {
        face
          .pairs()
          .into_par_iter()
          .map(move |(a, b)| ((a, b), i as u32))
      })
      .collect::<HashMap<_, _>>();

    let opposites = self
      .faces
      .par_iter()
      .map(|face| {
        face
          .pairs()
          .into_par_iter()
          .map(|(a, b)| {
            let arc = (b, a);
            arc_to_face_map.get(&arc).copied()
          })
          .collect::<Vec<_>>()
          .try_into()
          .unwrap()
      })
      .collect::<Vec<_>>();

    opposites
  }
}
