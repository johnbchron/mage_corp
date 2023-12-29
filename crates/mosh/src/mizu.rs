//! # Mizu
//! Mizu is an intelligent mesh data structure built for mesh simplification.

mod buffers;
mod face;
mod vertex;

use std::sync::OnceLock;

use hashbrown::{HashMap, HashSet};
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
  opposites: OnceLock<Vec<[Option<(u32, u8)>; 3]>>,
}

struct SimplificationCandidate {
  faces_to_remove: [u32; 4],
  new_face:        Face,
}

impl<D: VertexData> MizuMesh<D> {
  /// Returns a reference to the vertex at the given index.
  pub fn vertex(&self, index: u32) -> &Vertex<D> {
    &self.vertices[index as usize]
  }
  /// Returns a reference to the face at the given index.
  pub fn face(&self, index: u32) -> &Face { &self.faces[index as usize] }

  fn is_simplifiable_face(
    &self,
    index: u32,
  ) -> Option<SimplificationCandidate> {
    let neighbors = self
      .neighbors(index)
      .iter()
      .filter_map(|n| *n)
      .collect::<Vec<_>>();
    // if not all neighbors exist, abort
    if neighbors.len() != 3 {
      return None;
    }
    // if any of the neighbors are not coplanar, abort
    if neighbors.iter().any(|n| {
      let neighbor = self.face(n.0);
      !self.face(index).are_coplanar(neighbor)
    }) {
      return None;
    }

    let self_a = self.vertex(self.face(index).vertices().x);
    let self_b = self.vertex(self.face(index).vertices().y);
    let self_c = self.vertex(self.face(index).vertices().z);

    // get outside edges
    // v_o_1e = vertex_opposite_first_edge
    let v_o_1e = self
      .face(neighbors[0].0)
      .vertex_index_opposite_pair(neighbors[0].1);
    let v_o_2e = self
      .face(neighbors[1].0)
      .vertex_index_opposite_pair(neighbors[1].1);
    let v_o_3e = self
      .face(neighbors[2].0)
      .vertex_index_opposite_pair(neighbors[2].1);

    if Vertex::are_collinear(self.vertex(v_o_1e), self_a, self.vertex(v_o_3e))
      && Vertex::are_collinear(self.vertex(v_o_2e), self_b, self.vertex(v_o_1e))
      && Vertex::are_collinear(self.vertex(v_o_3e), self_c, self.vertex(v_o_2e))
    {
      return Some(SimplificationCandidate {
        faces_to_remove: [
          index,
          neighbors[0].0,
          neighbors[1].0,
          neighbors[2].0,
        ],
        new_face:        Face::new(
          glam::UVec3::new(v_o_1e, v_o_2e, v_o_3e),
          self.compute_normal(&glam::UVec3::new(v_o_1e, v_o_2e, v_o_3e)),
        ),
      });
    }
    None
  }

  fn apply_simplification_candidates(
    &mut self,
    mut candidates: Vec<SimplificationCandidate>,
  ) {
    let _span =
      info_span!("mosh::MizuMesh::apply_simplification_candidates").entered();

    let mut to_remove = HashSet::new();
    candidates.retain(|c| {
      let mut can_remove = true;
      for face in c.faces_to_remove.iter() {
        if to_remove.contains(face) {
          can_remove = false;
          break;
        }
      }
      if can_remove {
        for face in c.faces_to_remove.iter() {
          to_remove.insert(*face);
        }
      }
      can_remove
    });

    let mut to_remove = to_remove.into_iter().collect::<Vec<_>>();
    to_remove.sort_unstable();
    to_remove.reverse();

    for i in to_remove {
      self.faces.remove(i as usize);
    }

    for candidate in candidates {
      self.faces.push(candidate.new_face);
    }

    self.opposites.take();
  }

  fn prune_vertices(&mut self) {
    let _span = info_span!("mosh::MizuMesh::prune_vertices").entered();

    let mut vertex_map = HashMap::new();
    let mut new_vertices = Vec::new();

    for face in self.faces.iter() {
      for vertex in face.vertices().to_array().into_iter() {
        if !vertex_map.contains_key(&vertex) {
          vertex_map.insert(vertex, new_vertices.len() as u32);
          new_vertices.push(self.vertices[vertex as usize].clone());
        }
      }
    }

    for face in self.faces.iter_mut() {
      let vertices = face.vertices_mut();
      vertices.x = vertex_map[&vertices.x];
      vertices.y = vertex_map[&vertices.y];
      vertices.z = vertex_map[&vertices.z];
    }

    self.vertices = new_vertices;
    self.opposites.take();
  }

  /// Simplifies the mesh.
  pub fn simplify(&mut self) {
    let _span = info_span!("mosh::MizuMesh::simplify").entered();

    let _sub_span =
      info_span!("mosh::MizuMesh::simplify::get_candidates").entered();
    let candidates = (0..self.faces.len())
      .par_bridge()
      .filter_map(|i| self.is_simplifiable_face(i as u32))
      .collect::<Vec<_>>();
    drop(_sub_span);

    self.apply_simplification_candidates(candidates);
    self.prune_vertices();
  }

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
      .iter()
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
