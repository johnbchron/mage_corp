//! # Mizu
//! Mizu is an intelligent mesh data structure built for mesh simplification.

mod buffers;
mod face;
mod simplify;
mod vertex;

use std::sync::OnceLock;

use hashbrown::HashMap;
use rayon::prelude::*;
use tracing::info_span;

pub use self::{
  face::Face,
  vertex::{Vertex, VertexData},
};

type OppositeEntry = Option<(u32, u8)>;

/// A mesh data structure for mesh simplification.
pub struct MizuMesh<D: VertexData> {
  vertices:  Vec<Vertex<D>>,
  faces:     Vec<Face>,
  opposites: OnceLock<Vec<[OppositeEntry; 3]>>,
}

impl<D: VertexData> MizuMesh<D> {
  /// Returns a reference to the vertex at the given index.
  pub fn vertex(&self, index: u32) -> &Vertex<D> {
    &self.vertices[index as usize]
  }
  /// Returns a reference to the face at the given index.
  pub fn face(&self, index: u32) -> &Face { &self.faces[index as usize] }

  /// Computes the normal of the face with the given indices.
  pub fn compute_normal(&self, indices: &glam::UVec3) -> glam::Vec3A {
    let a = self.vertex(indices.x).pos();
    let b = self.vertex(indices.y).pos();
    let c = self.vertex(indices.z).pos();
    (b - a).cross(c - a).normalize()
  }

  /// Returns the neighbors of a face.
  pub fn neighbors(&self, index: u32) -> [Option<(u32, u8)>; 3] {
    self.opposites()[index as usize]
  }

  /// Returns true if the face at the given index neighbors the face at the
  /// given test index.
  pub fn neighbors_contain(&self, index: u32, test: u32) -> bool {
    self
      .neighbors(index)
      .iter()
      .any(|n| n.map(|n| n.0) == Some(test))
  }

  /// Returns true if the face at the given index neighbors any of the faces at
  /// the test indices.
  pub fn neighbors_contain_any(
    &self,
    index: u32,
    tested: impl IntoIterator<Item = u32>,
  ) -> bool {
    let tested = tested.into_iter().collect::<Vec<_>>();
    self.neighbors(index).iter().any(|n| {
      n.map(|n| tested.iter().any(|neighbor| *neighbor == n.0))
        .unwrap_or(false)
    })
  }

  /// Creates a new face using vertex info but does not add it to the mesh.
  pub fn create_face(
    &self,
    vertex_0: u32,
    vertex_1: u32,
    vertex_2: u32,
  ) -> Face {
    let vertices = glam::UVec3::new(vertex_0, vertex_1, vertex_2);
    let normal = self.compute_normal(&vertices);
    Face::new(vertices, normal)
  }

  /// Returns a slice of the opposites of each face in the mesh.
  pub fn opposites(&self) -> &[[Option<(u32, u8)>; 3]] {
    self.opposites.get_or_init(|| self.build_opposites())
  }

  fn build_opposites(&self) -> Vec<[Option<(u32, u8)>; 3]> {
    let _span = info_span!("mosh::MizuMesh::build_opposites").entered();

    let arc_to_face_map = self
      .faces
      .iter()
      .enumerate()
      .flat_map(|(i, face)| {
        (0..3).map(move |j| (face.pair(j as u8), (i as u32, j as u8)))
      })
      .collect::<HashMap<_, _>>();

    let opposites = self
      .faces
      .par_iter()
      .map(|face| {
        face
          .pairs()
          .into_iter()
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
